use database::Record;
use kdlize::{AsKdl, FromKdl};
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
impl AsKdl for ListId {
	fn as_kdl(&self) -> kdlize::NodeBuilder {
		kdlize::NodeBuilder::default()
			.with_entry(self.owner.as_str())
			.with_entry(self.id.as_str())
	}
}
impl FromKdl<()> for ListId {
	type Error = kdlize::error::Error;

	fn from_kdl<'doc>(node: &mut kdlize::NodeReader<'doc, ()>) -> Result<Self, Self::Error> {
		let owner = node.next_str_req()?.to_owned();
		let id = node.next_str_req()?.to_owned();
		Ok(Self { owner, id })
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct List {
	pub id: ListId,
	pub file_id: Option<String>,
	pub kdl: String,
	pub name: String,
	pub local_version: String,
	pub remote_version: String,
}

impl Record for List {
	fn store_id() -> &'static str {
		"lists"
	}
}
