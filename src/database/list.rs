use std::{path::Path, str::FromStr};

use database::Record;
use kdlize::{AsKdl, FromKdl};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ListId {
	pub owner: String,
	pub id: String,
}
impl std::fmt::Debug for ListId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "ListId({}/{})", self.owner, self.id)
	}
}
impl ToString for ListId {
	fn to_string(&self) -> String {
		format!("{}/{}", self.owner, self.id)
	}
}
#[derive(thiserror::Error, Debug, Clone)]
pub enum ListIdParserError {
	#[error("Missing owner in id {0:?}")]
	MissingOwner(String),
	#[error("Missing name in id {0:?}")]
	MissingName(String),
}
impl FromStr for ListId {
	type Err = ListIdParserError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut iter = s.split("/");
		let owner = iter.next().ok_or(ListIdParserError::MissingOwner(s.to_owned()))?;
		let id = iter.next().ok_or(ListIdParserError::MissingName(s.to_owned()))?;
		Ok(Self {
			owner: owner.to_owned(),
			id: id.to_owned(),
		})
	}
}

impl ListId {
	pub fn from_path(owner: impl Into<String>, path: impl AsRef<Path>) -> Self {
		let stem_os = path.as_ref().file_stem().unwrap();
		let stem_str = stem_os.to_str().unwrap();
		let id = stem_str.to_owned();
		Self {
			owner: owner.into(),
			id,
		}
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

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct List {
	pub id: String,
	pub file_id: String,
	pub kdl: String,
	pub local_version: String,
	pub pending_changes: Vec<(String, String)>,
}

impl Record for List {
	fn store_id() -> &'static str {
		"lists"
	}

	fn key(&self) -> Option<&String> {
		Some(&self.id)
	}
}
