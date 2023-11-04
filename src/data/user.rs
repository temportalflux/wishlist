use crate::database::ListId;
use kdlize::{AsKdl, FromKdl};

#[derive(Clone, PartialEq, Default)]
pub struct User {
	// wishlists owned by others that the user has accepted invites to
	pub external_lists: Vec<ListId>,
}

impl AsKdl for User {
	fn as_kdl(&self) -> kdlize::NodeBuilder {
		let mut node = kdlize::NodeBuilder::default();
		for list_id in &self.external_lists {
			node.push_child_t("external_list", list_id);
		}
		node
	}
}
impl FromKdl<()> for User {
	type Error = anyhow::Error;

	fn from_kdl<'doc>(node: &mut kdlize::NodeReader<'doc, ()>) -> Result<Self, Self::Error> {
		let external_lists = node.query_all_t::<ListId>("scope() > external_list")?;
		Ok(Self { external_lists })
	}
}
