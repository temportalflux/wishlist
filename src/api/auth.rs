use crate::api::github::AuthStatus;
use serde::Deserialize;
use std::collections::HashMap;
use yew::{html, Html};
use yew_router::Routable;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
	#[at("/api/auth/login")]
	Login,
	#[at("/api/auth/logout")]
	Logout,
	#[at("/api/auth/login_token")]
	TokenExchange,
}

impl crate::route::Route for Route {
	fn html(self) -> Html {
		let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();
		match (self, AuthStatus::load()) {
			(Self::Login, None) => {
				AuthStatus::Authorizing.apply_to_session();
				let auth_url = {
					let mut params = HashMap::new();
					params.insert("client_id", crate::config::CLIENT_ID.to_string());
					params.insert("scope", "gist".to_string());
					params.insert("redirect_uri", format!("{base_url}api/auth/login_token"));
					let params = params
						.into_iter()
						.map(|(k, v)| format!("{k}={v}"))
						.collect::<Vec<_>>()
						.join("&");
					format!("https://github.com/login/oauth/authorize?{params}")
				};
				if let Err(err) = gloo_utils::window().location().replace(auth_url.as_str()) {
					AuthStatus::Failed(format!("{err:?}")).apply_to_session();
				}
			}
			(Self::Logout, Some(_)) => {
				AuthStatus::delete();
				let _ = gloo_utils::window().location().replace(&base_url);
			}
			(Self::TokenExchange, Some(AuthStatus::Authorizing)) => {
				AuthStatus::ExchangingTokens.apply_to_session();
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
							AuthStatus::Failed(format!("{err:?}")).apply_to_session();
							return;
						}
					};
					let response_text = match response.text().await {
						Ok(data) => data,
						Err(err) => {
							AuthStatus::Failed(format!("{err:?}")).apply_to_session();
							return;
						}
					};
					let data: AccessTokenResponse = match serde_json::from_str(&response_text) {
						Ok(data) => data,
						Err(_) => {
							AuthStatus::Failed(format!("Not an access token: {response_text:?}")).apply_to_session();
							return;
						}
					};

					AuthStatus::Successful(data.access_token).apply_to_session();

					let _ = gloo_utils::window().location().replace(&base_url);
				});
			}
			_ => {}
		}
		html! {
			<crate::index::Page />
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
