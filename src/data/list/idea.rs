use kdlize::{ext::DocumentExt, AsKdl, FromKdl};

/// Represents a generic idea or concept for a gift.
/// Purchasers will need to find the actual gift themselves.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Idea {
	pub image_url: Option<String>,
	pub estimated_cost: usize,
	pub example_urls: Vec<String>,
}

impl FromKdl<()> for Idea {
	type Error = kdlize::error::Error;

	fn from_kdl<'doc>(node: &mut kdlize::NodeReader<'doc, ()>) -> Result<Self, Self::Error> {
		let image_url = node.query_str_opt("scope() > image", 0)?.map(str::to_owned);
		let estimated_cost = node.query_i64_req("scope() > estimated_cost", 0)? as usize;
		let example_urls = node.query_str_all("scope() > example", 0)?;
		let example_urls = example_urls.into_iter().map(str::to_owned).collect();
		Ok(Self {
			image_url,
			estimated_cost,
			example_urls,
		})
	}
}

impl AsKdl for Idea {
	fn as_kdl(&self) -> kdlize::NodeBuilder {
		let mut node = kdlize::NodeBuilder::default();
		if let Some(url) = &self.image_url {
			node.push_child_t("image", url);
		}
		node.push_child_entry("estimated_cost", self.estimated_cost as i64);
		for url in &self.example_urls {
			node.push_child_t("example", url);
		}
		node
	}
}
