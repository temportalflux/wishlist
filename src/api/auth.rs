use crate::{
	api::github::{gist::FetchProfile, FetchCurrentUser},
	response::InvalidJson,
	session::{AuthStatus, Profile, User},
};
use serde::Deserialize;
use std::collections::HashMap;
use yew::{function_component, html, Html};
use yew_hooks::{use_async, use_is_first_mount};
use yew_router::Routable;
use yewdux::prelude::Dispatch;

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
		let auth_status = Dispatch::<AuthStatus>::new();
		match (self, auth_status.get().as_ref()) {
			(Self::Login, AuthStatus::None) => {
				let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();
				auth_status.set(AuthStatus::Authorizing);
				let auth_url = {
					let mut params = HashMap::new();
					params.insert("client_id", crate::config::CLIENT_ID.trim().to_string());
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
					auth_status.set(AuthStatus::Failed(format!("{err:?}")));
				}
			}
			(Self::Logout, status) if *status != AuthStatus::None => {
				let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();
				auth_status.set(AuthStatus::None);
				Dispatch::<User>::new().set(User::default());
				Dispatch::<Profile>::new().set(Profile::default());
				let _ = gloo_utils::window().location().replace(&base_url);
			}
			(Self::TokenExchange, AuthStatus::Authorizing) => {
				return html! { <TokenExchangePage /> }
			}
			_ => {}
		}
		html! { <crate::page::Page /> }
	}
}

#[function_component]
fn TokenExchangePage() -> Html {
	let auth_status = Dispatch::<AuthStatus>::new();
	auth_status.set(AuthStatus::ExchangingTokens);

	let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();
	let params_str = gloo_utils::window().location().search().unwrap();
	let params = web_sys::UrlSearchParams::new_with_str(&params_str).unwrap();
	let code = params.get("code").unwrap();

	let authenticate = use_async(async move {
		static MAX_ATTEMPTS: usize = 2;
		let mut attempt = 0;
		'attempt_exchange: while attempt < MAX_ATTEMPTS {
			match exchange_tokens(&code).await {
				Ok(token) => {
					auth_status.set(AuthStatus::Successful(token));
					break 'attempt_exchange;
				}
				Err(err) => {
					auth_status.set(AuthStatus::Failed(format!("{err:?}")));
					attempt += 1;
				}
			}
		}

		// Ran out of attempts and still didn't succeed
		if attempt >= MAX_ATTEMPTS {
			return Ok(());
		}

		match FetchCurrentUser::get().await {
			Ok(data) => Dispatch::<User>::new().set(data),
			Err(err) => log::debug!("{err:?}"),
		}
		match FetchProfile::get().await {
			Ok(profile) => Dispatch::<Profile>::new().set(profile),
			Err(err) => log::debug!("{err:?}"),
		}

		let _ = gloo_utils::window().location().replace(&base_url);
		Ok(()) as Result<(), ()>
	});

	if use_is_first_mount() {
		authenticate.run();
	}

	html! {
		<crate::page::Page />
	}
}

async fn exchange_tokens(code: &String) -> anyhow::Result<String> {
	/* Github OAuth & CORS issue
	- Explanation: https://stackoverflow.com/questions/42150075/cors-issue-on-github-oauth
	- Reverse Proxy: https://stackoverflow.com/questions/29670703/how-to-use-cors-anywhere-to-reverse-proxy-and-add-cors-headers/32167044#32167044
	- Cors-Anywhere Deprecation: https://github.com/Rob--W/cors-anywhere/issues/301
	- cors.sh blog post: https://blog.grida.co/cors-anywhere-for-everyone-free-reliable-cors-proxy-service-73507192714e
	- [https://cors.sh/]
	*/
	let payload = {
		let mut payload = std::collections::HashMap::new();
		payload.insert("client_id", crate::config::CLIENT_ID.trim());
		payload.insert("client_secret", crate::config::CLIENT_SECRET.trim());
		payload.insert("code", &code);
		payload
	};
	// Proxies:
	//let proxy = "https://proxy.cors.sh/";
	//let proxy = "https://thingproxy.freeboard.io/fetch/";
	let proxy = "https://corsproxy.io/?";
	let target = "https://github.com/login/oauth/access_token";
	let target = urlencoding::encode(target);
	let builder = reqwest::Client::new()
		.post(format!("{proxy}{target}"))
		.json(&payload)
		.headers({
			let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();
			let mut header = reqwest::header::HeaderMap::new();
			header.insert("origin", base_url.parse().unwrap());
			header.insert("Accept", "application/json".parse().unwrap());
			header.insert("Content-Type", "application/json".parse().unwrap());
			header
		});
	log::debug!("Request: {:?}, Payload: {:?}", builder, payload);
	let response = builder.send().await?;
	let text = response.text().await?;
	let data: AccessTokenResponse = match serde_json::from_str(&text) {
		Ok(data) => data,
		Err(err) => {
			return Err(InvalidJson(text, err))?;
		}
	};
	Ok(data.access_token)
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
