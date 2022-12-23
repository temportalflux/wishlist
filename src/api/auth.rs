use crate::api::github::AccessToken;
use serde::Deserialize;
use ybc::{Container, Title};
use yew::{html, Html};
use yew_router::Routable;

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
		let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();
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
			Self::Logout => {
				AccessToken::delete();
				let _ = gloo_utils::window().location().replace(&base_url);
				html! {"logging out"}
			}
			Self::TokenExchange => {
				let params_str = gloo_utils::window().location().search().unwrap();
				let params = web_sys::UrlSearchParams::new_with_str(&params_str).unwrap();
				let code = params.get("code").unwrap();

				wasm_bindgen_futures::spawn_local(async move {
					/* Github OAuth & CORS issue
					- Explanation: https://stackoverflow.com/questions/42150075/cors-issue-on-github-oauth
					- Reverse Proxy: https://stackoverflow.com/questions/29670703/how-to-use-cors-anywhere-to-reverse-proxy-and-add-cors-headers/32167044#32167044
					- Cors-Anywhere Deprecation: https://github.com/Rob--W/cors-anywhere/issues/301
					- cors.sh blog post: https://blog.grida.co/cors-anywhere-for-everyone-free-reliable-cors-proxy-service-73507192714e
					- [https://cors.sh/]
					*/
					let client = reqwest::Client::new();
					let res = client
						.post("https://proxy.cors.sh/https://github.com/login/oauth/access_token")
						.json(&{
							let mut data = std::collections::HashMap::new();
							data.insert("client_id", crate::config::CLIENT_ID);
							data.insert("client_secret", crate::config::CLIENT_SECRET);
							data.insert("code", &code);
							data
						})
						.headers({
							let mut header = reqwest::header::HeaderMap::new();
							header.insert("origin", "https://localhost:8080".parse().unwrap());
							header.insert("Accept", "application/json".parse().unwrap());
							header.insert("Content-Type", "application/json".parse().unwrap());
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
					let response_text = match response.text().await {
						Ok(data) => data,
						Err(err) => {
							log::error!("{err:?}");
							return;
						}
					};
					let data: AccessTokenResponse = match serde_json::from_str(&response_text) {
						Ok(data) => data,
						Err(err) => {
							log::error!("{err:?}");
							return;
						}
					};

					AccessToken::from(data.access_token).save();

					let _ = gloo_utils::window().location().replace(&base_url);
				});
				html! {
					<Container>
						<Title>{"Establishing Authentication Handshake"}</Title>
						<progress class={"progress is-large is-info"}></progress>
					</Container>
				}
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
