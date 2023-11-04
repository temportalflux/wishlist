use super::Kind;
use kdlize::{ext::DocumentExt, AsKdl, FromKdl};
use std::collections::{BTreeMap, BTreeSet};

/// A wish/gift idea on a wishlist.
#[derive(Clone, PartialEq, Debug)]
pub struct Entry {
	/// The display name
	pub name: String,
	/// Misc information the user has provided
	pub description: String,
	/// How many of this entry can be reserved.
	pub quantity: usize,
	pub reservations: BTreeMap<String, usize>,
	/// Categories that this item should show up under.
	pub tags: BTreeSet<String>,
	/// The item-subtype for additional properties.
	pub kind: Kind,
}
impl Default for Entry {
	fn default() -> Self {
		Self {
			name: Default::default(),
			description: Default::default(),
			quantity: 1,
			reservations: Default::default(),
			tags: Default::default(),
			kind: Default::default(),
		}
	}
}

impl FromKdl<()> for Entry {
	type Error = kdlize::error::Error;

	fn from_kdl<'doc>(node: &mut kdlize::NodeReader<'doc, ()>) -> Result<Self, Self::Error> {
		let name = node.next_str_req()?.to_owned();
		let description = node.query_str_req("scope() > description", 0)?.to_owned();

		let quantity = node.query_i64_req("scope() > quantity", 0)? as usize;
		let mut reservations = BTreeMap::new();
		for mut node in node.query_all("scope() > quantity > reservation")? {
			let user_id = node.next_str_req()?.to_owned();
			let amount = node.next_i64_req()? as usize;
			reservations.insert(user_id, amount);
		}

		let tags = node.query_str_all("scope() > tag", 0)?;
		let tags = tags.into_iter().map(str::to_owned).collect();

		let kind = node.query_req_t("scope() > kind")?;

		Ok(Self {
			name,
			description,
			quantity,
			reservations,
			tags,
			kind,
		})
	}
}

impl AsKdl for Entry {
	fn as_kdl(&self) -> kdlize::NodeBuilder {
		let mut node = kdlize::NodeBuilder::default();
		node.push_entry(self.name.as_str());
		node.push_child_entry("description", self.description.as_str());
		node.push_child({
			let mut node = kdlize::NodeBuilder::default();
			node.push_entry(self.quantity as i64);
			for (user_id, amount) in &self.reservations {
				node.push_child(
					kdlize::NodeBuilder::default()
						.with_entry(user_id.as_str())
						.with_entry(*amount as i64)
						.build("reservation"),
				);
			}
			node.build("quantity")
		});
		for tag in &self.tags {
			node.push_child_entry("tag", tag.as_str());
		}
		node.push_child_t("kind", &self.kind);
		node
	}
}
