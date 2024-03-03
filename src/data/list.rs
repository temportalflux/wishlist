use kdlize::{ext::DocumentExt, AsKdl, FromKdl};

mod entry;
pub use entry::*;
mod kind;
pub use kind::*;
mod specific;
pub use specific::*;
mod idea;
pub use idea::*;
mod bundle;
pub use bundle::*;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct List {
	pub name: String,
	// user-ids of those whove been invited to access this list (in addition to the owner)
	pub invitees: Vec<String>,
	pub entries: Vec<Entry>,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum RecordError {
	#[error(transparent)]
	Database(#[from] database::Error),
	#[error(transparent)]
	Parsing(#[from] kdl::KdlError),
	#[error("{0}")]
	Interpretting(String),
}
impl From<anyhow::Error> for RecordError {
	fn from(value: anyhow::Error) -> Self {
		Self::Interpretting(format!("{value:?}"))
	}
}
impl From<kdlize::error::Error> for RecordError {
	fn from(value: kdlize::error::Error) -> Self {
		Self::Interpretting(format!("{value:?}"))
	}
}

impl super::RecordData for List {
	type Record = crate::database::List;
	type Error = RecordError;

	fn parse_record(record: &Self::Record) -> Result<Self, Self::Error> {
		let document = record.kdl.parse::<kdl::KdlDocument>()?;
		let Some(node) = document.nodes().get(0) else {
			use kdlize::error::*;
			return Err(Error::from(EmptyDocument(document)).into());
		};
		let mut reader = kdlize::NodeReader::new_root(node, ());
		Ok(Self::from_kdl(&mut reader)?)
	}
}

kdlize::impl_kdl_node!(List, "list");

impl FromKdl<()> for List {
	type Error = anyhow::Error;

	fn from_kdl<'doc>(node: &mut kdlize::NodeReader<'doc, ()>) -> Result<Self, Self::Error> {
		let name = node.next_str_req()?.to_owned();
		let invitees = node.query_str_all("scope() > invite", 0)?;
		let invitees = invitees.into_iter().map(str::to_owned).collect();
		let entries = node.query_all_t("scope() > entry")?;
		Ok(Self {
			name,
			invitees,
			entries,
		})
	}
}

impl AsKdl for List {
	fn as_kdl(&self) -> kdlize::NodeBuilder {
		let mut node = kdlize::NodeBuilder::default();
		node.push_entry(self.name.as_str());
		for invitee in &self.invitees {
			node.push_child_entry("invite", invitee.as_str());
		}
		node.push_children_t(("entry", self.entries.iter()));
		node
	}
}
