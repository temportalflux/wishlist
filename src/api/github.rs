use gloo_storage::{SessionStorage, Storage};
use reqwest::{Method, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug)]
pub struct Session {
	pub status: Option<AuthStatus>,
	pub user: Option<User>,
}
impl Session {
	pub fn get() -> Self {
		Self {
			status: AuthStatus::load(),
			user: User::load(),
		}
	}

	pub fn delete() {
		AuthStatus::delete();
		User::delete();
	}
}

pub trait SessionValue {
	fn id() -> &'static str;

	fn load() -> Option<Self>
	where
		Self: for<'de> Deserialize<'de>,
	{
		SessionStorage::get::<Self>(Self::id()).ok()
	}

	fn apply_to_session(self)
	where
		Self: Sized + Serialize,
	{
		let _ = SessionStorage::set(Self::id(), self);
	}

	fn delete() {
		SessionStorage::delete(Self::id());
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthStatus {
	Authorizing,
	ExchangingTokens,
	Successful(String),
	Failed(String),
}
impl SessionValue for AuthStatus {
	fn id() -> &'static str {
		"auth_status"
	}
}
impl AuthStatus {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
	pub name: String,
	pub login: String,
	pub image_url: String,
}
impl SessionValue for User {
	fn id() -> &'static str {
		"user"
	}
}

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

pub fn query_rest<D, T>(method: Method, endpoint: &str, content: Option<&D>) -> Response<T>
where
	D: Serialize + ?Sized,
	T: DeserializeOwned,
{
	let AuthStatus::Successful(token) = AuthStatus::load().unwrap() else {
		unimplemented!("No auth token while building request")
	};
	let endpoint = format!("https://api.github.com/{endpoint}");
	let mut builder = reqwest::Client::new().request(method, &endpoint);
	builder = builder.header("Authorization", format!("Bearer {token}"));
	builder = builder.header("Accept", "application/vnd.github+json");
	builder = builder.header("Content-Type", "application/json");
	builder = builder.header("X-GitHub-Api-Version", "2022-11-28");
	if let Some(data) = content {
		builder = builder.json(&data);
	}
	Response::<T>::from(builder)
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
