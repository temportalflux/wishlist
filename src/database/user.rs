use database::Record;
use serde::{Deserialize, Serialize};
use super::ListId;

// Represents a person who has wishlists. This may be the logged in user, someone whose invited them to a wishlist, or someone the user has invited.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct User {
	pub login: String,
	pub file_id: Option<String>,
	pub kdl: String,
	// wishlists owned by others that the user has been invited to
	pub external_invites: Vec<ListId>,
}

impl Record for User {
	fn store_id() -> &'static str {
		"users"
	}
}
