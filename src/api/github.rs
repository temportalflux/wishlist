use gloo_storage::{SessionStorage, Storage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessToken(String);
impl From<String> for AccessToken {
	fn from(value: String) -> Self {
		Self(value)
	}
}
impl AccessToken {
	fn id() -> &'static str {
		"access_token"
	}

	pub fn load() -> Option<Self> {
		SessionStorage::get::<Self>(Self::id()).ok()
	}

	pub fn save(self) {
		let _ = SessionStorage::set(Self::id(), self);
	}

	pub fn delete() {
		SessionStorage::delete(Self::id());
	}
}
