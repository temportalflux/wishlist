use gloo_storage::{SessionStorage, Storage};

#[derive(Debug)]
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
		SessionStorage::get::<String>(Self::id()).ok().map(Self)
	}

	pub fn save(self) {
		let _ = SessionStorage::set(Self::id(), self.0);
	}

	pub fn delete() {
		SessionStorage::delete(Self::id());
	}
}
