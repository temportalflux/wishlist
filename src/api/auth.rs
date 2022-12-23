use yew::{html, Html};
use yew_router::Routable;
use serde::Deserialize;
use crate::api::github::AccessToken;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
	#[at("api/auth/login")]
	Login,
	#[at("api/auth/logout")]
	Logout,
	#[at("api/auth/login_token")]
	TokenExchange,
}

impl crate::route::Route for Route {
	fn html(&self) -> Html {
		match self {
			Self::Login => {
				let auth_url = {
					let mut url = "https://github.com/login/oauth/authorize".to_string();
					url += &format!("?client_id={}", crate::config::CLIENT_ID);
					url += "&scope=gist";
					url += "&redirect_uri=http://localhost:8080/api/auth/login_token";
					url
				};
				let _ = gloo_utils::window().location().replace(auth_url.as_str());
				html! {"logging in"}
			}
			Self::Logout => html! {
				{"logout"}
			},
			Self::TokenExchange => {
				let params_str = gloo_utils::window().location().search().unwrap();
				let params = web_sys::UrlSearchParams::new_with_str(&params_str).unwrap();
				let code = params.get("code").unwrap();
				let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();

				wasm_bindgen_futures::spawn_local(async move {
					let client = reqwest::Client::new();
					let res = client.post("https://cors-anywhere.herokuapp.com/https://github.com/login/oauth/access_token")
							.query(&[
								("client_id", crate::config::CLIENT_ID),
								("client_secret", crate::config::CLIENT_SECRET),
								("code", &code),
							])
							.headers({
								let mut header = reqwest::header::HeaderMap::new();
								header.insert("Accept", "application/json".parse().unwrap());
								header.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());
								header
							})
							.send()
							.await;
					let response = match res {
						Ok(resp) => resp,
						Err(err) => {
							log::error!("{err:?}");
							return;
						}
					};
					let data: AccessTokenResponse = match response.json().await {
						Ok(data) => data,
						Err(err) => {
							log::error!("{err:?}");
							return;
						}
					};

					AccessToken::from(data.access_token).save();

					let _ = gloo_utils::window().location().replace(&base_url);
				});
				html! {"exchanging auth token"}
			}
		}
	}
}

#[derive(Deserialize)]
struct AccessTokenResponse {
	access_token: String,
}

impl yew::html::IntoPropValue<Option<String>> for Route {
	fn into_prop_value(self) -> Option<String> {
		Some(self.to_path())
	}
}
