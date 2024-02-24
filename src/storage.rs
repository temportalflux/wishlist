pub mod autosync;

pub static USER_DATA_REPO_NAME: &str = "wishlist-app-data";
pub static DATA_REPO_TOPIC: &str = "wishlist-app-data";
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub fn get(status: &crate::auth::Status) -> Option<github::GithubClient> {
	let crate::auth::Status::Successful { oauth_id: _, token } = status else {
		return None;
	};
	github::GithubClient::new(token, APP_USER_AGENT).ok()
}
