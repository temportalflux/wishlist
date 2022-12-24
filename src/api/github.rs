use gloo_storage::{SessionStorage, Storage};
use serde::{Deserialize, Serialize};

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
