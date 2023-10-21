use database::Record;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListId {
	pub owner: String,
	pub id: String,
}
impl std::fmt::Debug for ListId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "ListId({}/{})", self.owner, self.id)
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct List {
	pub id: ListId,
	pub file_id: Option<String>,
	pub kdl: String,
	pub name: String,
	pub version: String,
	pub remote_version: String,
	// user-ids of those whove been invited to access this list (in addition to the owner)
	pub invitees: Vec<String>,
}

impl Record for List {
	fn store_id() -> &'static str {
		"lists"
	}
}
