use kdlize::{ext::DocumentExt, AsKdl, FromKdl};

/// Represents a specific gift.
/// These can be purchased directly without any additional considerations from the purchaser.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Specific {
	pub image_url: Option<String>,
	pub offer_url: String,
	pub cost_per_unit: usize,
}

impl FromKdl<()> for Specific {
	type Error = kdlize::error::Error;

	fn from_kdl<'doc>(node: &mut kdlize::NodeReader<'doc, ()>) -> Result<Self, Self::Error> {
		let image_url = node.query_str_opt("scope() > image", 0)?.map(str::to_owned);
		let offer_url = node.query_str_req("scope() > offer", 0)?.to_owned();
		let cost_per_unit = node.query_i64_req("scope() > cost_per_unit", 0)? as usize;
		Ok(Self {
			image_url,
			offer_url,
			cost_per_unit,
		})
	}
}

impl AsKdl for Specific {
	fn as_kdl(&self) -> kdlize::NodeBuilder {
		let mut node = kdlize::NodeBuilder::default();
		if let Some(url) = &self.image_url {
			node.push_child_t("image", url);
		}
		node.push_child_entry("offer", self.offer_url.as_str());
		node.push_child_entry("cost_per_unit", self.cost_per_unit as i64);
		node
	}
}
