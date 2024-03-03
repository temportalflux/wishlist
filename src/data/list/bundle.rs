use super::Entry;
use kdlize::{AsKdl, FromKdl};

/// Represent a group/set of items that should be purchased together.
/// Any item in the bundle can be a specific or idea item, but there shouldn't be interior bundling (this is not enforced).
/// Example: a specific poster and an idea for a frame for the poster.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Bundle {
	pub entries: Vec<Entry>,
}

impl FromKdl<()> for Bundle {
	type Error = anyhow::Error;

	fn from_kdl<'doc>(node: &mut kdlize::NodeReader<'doc, ()>) -> Result<Self, Self::Error> {
		let entries = node.query_all_t("scope() > entry")?;
		Ok(Self { entries })
	}
}

impl AsKdl for Bundle {
	fn as_kdl(&self) -> kdlize::NodeBuilder {
		let mut node = kdlize::NodeBuilder::default();
		node.push_children_t(("entry", self.entries.iter()));
		node
	}
}
