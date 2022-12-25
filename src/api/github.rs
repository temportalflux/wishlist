use gloo_storage::{SessionStorage, Storage};
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthStatus {
	Authorizing,
	ExchangingTokens,
	Successful(String),
	Failed(String),
}
impl AuthStatus {
	fn id() -> &'static str {
		"auth_status"
	}

	pub fn load() -> Option<Self> {
		SessionStorage::get::<Self>(Self::id()).ok()
	}

	pub fn apply_to_session(self) {
		let _ = SessionStorage::set(Self::id(), self);
	}

	pub fn delete() {
		SessionStorage::delete(Self::id());
	}

	pub fn should_show_modal(&self) -> bool {
		match self {
			Self::Authorizing | Self::ExchangingTokens => true,
			Self::Successful(_) => false,
			Self::Failed(_) => true,
		}
	}

	pub fn should_show_progress(&self) -> bool {
		match self {
			Self::Failed(_) => false,
			_ => true,
		}
	}

	pub fn byline(&self) -> &'static str {
		match self {
			Self::Authorizing => "Establishing handshake",
			Self::ExchangingTokens => "Shaking hands",
			Self::Successful(_) => "Greetings completed",
			Self::Failed(_) => "Failed to authenticate",
		}
	}

	pub fn info(&self) -> Option<String> {
		match self {
			Self::Failed(error) => Some(error.clone()),
			_ => None,
		}
	}
}

pub struct Query;
impl Query {
	pub fn new<T>(query_string: &str) -> Response<T>
	where
		Self: Sized,
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
			data.insert("query", query_string);
			data
		});
		Response::<T>::from(builder)
	}
}

pub struct Response<T> {
	builder: RequestBuilder,
	marker: std::marker::PhantomData<T>,
}
impl<T> Response<T>
where
	T: DeserializeOwned,
{
	pub fn from(builder: RequestBuilder) -> Self {
		Self {
			builder,
			marker: Default::default(),
		}
	}

	pub async fn send(self) -> anyhow::Result<T> {
		let response: reqwest::Response = self.builder.send().await?;
		let text = response.text().await?;
		let output = match serde_json::from_str(&text) {
			Ok(data) => data,
			Err(err) => {
				return Err(InvalidJson(text, err))?;
			}
		};
		Ok(output)
	}
}

#[derive(thiserror::Error, Debug)]
pub struct InvalidJson(pub String, pub serde_json::Error);
impl std::fmt::Display for InvalidJson {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Invalid json: {:?}\nError: {:?}", self.0, self.1)
	}
}

pub struct CurrentUser;
impl CurrentUser {
	pub async fn get() -> anyhow::Result<String> {
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
		}
		let resp = Query::new::<Response>("query { viewer { login } }")
			.send()
			.await?;
		Ok(resp.data.viewer.login)
	}
}
