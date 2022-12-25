use gloo_storage::{SessionStorage, Storage};
use serde::{Deserialize, Serialize};

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
