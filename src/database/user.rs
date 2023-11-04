use database::Record;
use serde::{Deserialize, Serialize};

// Represents a person who has wishlists. This may be the logged in user, someone whose invited them to a wishlist, or someone the user has invited.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct User {
	pub login: String,
	pub file_id: String,
	pub kdl: String,
	// the tree-id of the root directory in the remote repository
	pub root_tree_id: String,
	// the version of the user's repository that is synced to locally
	pub local_version: String,
}

impl Record for User {
	fn store_id() -> &'static str {
		"users"
	}
}
