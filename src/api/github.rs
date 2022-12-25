use crate::{
	response::Response,
	session::{AuthStatus, SessionValue, User},
};
use reqwest::{Method, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod gist;

pub fn query_graphql<T>(query: &str) -> Response<T>
where
	T: DeserializeOwned,
{
	static ENDPOINT: &'static str = "https://api.github.com/graphql";
	let AuthStatus::Successful(token) = AuthStatus::load().unwrap() else {
		unimplemented!("No auth token while building request")
	};
	let mut builder = reqwest::Client::new().post(ENDPOINT);
	builder = builder.header("Authorization", format!("Bearer {token}"));
	builder = builder.header("Accept", "application/json");
	builder = builder.header("Content-Type", "application/json");
	builder = builder.json(&{
		let mut data = std::collections::HashMap::new();
		data.insert("query", query);
		data
	});
	Response::<T>::from(builder)
}

pub fn query_rest<T>(
	method: Method,
	endpoint: &str,
	add_body: Option<fn(RequestBuilder) -> RequestBuilder>,
) -> Response<T>
where
	T: DeserializeOwned,
{
	let AuthStatus::Successful(token) = AuthStatus::load().unwrap() else {
		unimplemented!("No auth token while building request")
	};
	let endpoint = format!("https://api.github.com{endpoint}");
	let mut builder = reqwest::Client::new().request(method, &endpoint);
	builder = builder.header("Authorization", format!("Bearer {token}"));
	builder = builder.header("Accept", "application/vnd.github+json");
	//builder = builder.header("Content-Type", "application/json");
	builder = builder.header("X-GitHub-Api-Version", "2022-11-28");
	if let Some(add_body) = add_body {
		builder = add_body(builder);
	}
	Response::<T>::from(builder)
}

pub struct FetchCurrentUser;
impl FetchCurrentUser {
	pub async fn get() -> anyhow::Result<User> {
		#[derive(Deserialize)]
		struct Response {
			data: Data,
		}
		#[derive(Deserialize)]
		struct Data {
			viewer: Viewer,
		}
		#[derive(Deserialize)]
		struct Viewer {
			login: String,
			name: String,
			#[serde(rename = "avatarUrl")]
			image_url: String,
		}
		let resp = query_graphql::<Response>("query { viewer { login name avatarUrl } }")
			.send()
			.await?;
		Ok(User {
			name: resp.data.viewer.name,
			login: resp.data.viewer.login,
			image_url: resp.data.viewer.image_url,
		})
	}
}
