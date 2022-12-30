use crate::api::github::gist::{GistId, GistInfo};
use serde::{Deserialize, Serialize};
use yewdux::store::Store;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "session", storage_tab_sync)]
pub enum AuthStatus {
	None,
	Authorizing,
	ExchangingTokens,
	Successful(String),
	Failed(String),
}
impl Default for AuthStatus {
	fn default() -> Self {
		Self::None
	}
}
impl AuthStatus {
	pub fn should_show_modal(&self) -> bool {
		match self {
			Self::Authorizing | Self::ExchangingTokens => true,
			Self::None | Self::Successful(_) => false,
			Self::Failed(_) => true,
		}
	}

	pub fn should_show_progress(&self) -> bool {
		match self {
			Self::None | Self::Failed(_) => false,
			_ => true,
		}
	}

	pub fn byline(&self) -> &'static str {
		match self {
			Self::None => "",
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "session", storage_tab_sync)]
pub struct User {
	pub name: String,
	pub login: String,
	pub image_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "session", storage_tab_sync)]
pub struct Profile {
	pub app_user_data: GistId,
	pub lists: Vec<GistInfo>,
}
