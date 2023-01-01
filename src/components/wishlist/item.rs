use serde::{Deserialize, Serialize};
use std::{collections::{BTreeSet, VecDeque}, str::FromStr};

pub trait ItemContainer {
	fn get_items(&self) -> Option<&Vec<Item>>;
	fn get_items_mut(&mut self) -> Option<&mut Vec<Item>>;
	
	fn get_item(&self, idx: usize) -> Option<&Item> {
		match self.get_items() {
			Some(items) => items.get(idx),
			None => None,
		}
	}
	
	fn get_item_mut(&mut self, idx: usize) -> Option<&mut Item> {
		match self.get_items_mut() {
			Some(items) => items.get_mut(idx),
			None => None,
		}
	}

	fn get_item_mut_at(&mut self, mut path: VecDeque<usize>) -> Option<&mut Item> {
		match path.pop_front() {
			Some(idx) => match self.get_item_mut(idx) {
				Some(item) => match path.is_empty() {
					true => Some(item),
					false => item.get_item_mut_at(path),
				}
				None => None,
			}
			None => None,
		}
	}

	fn get_items_for(&self, path: VecDeque<usize>) -> Vec<(VecDeque<usize>, &Item)> where Self: Sized {
		let mut container: &dyn ItemContainer = self;
		let mut items = Vec::with_capacity(path.len());
		let mut item_path = VecDeque::with_capacity(path.len());
		for idx in path {
			let Some(item) = container.get_item(idx) else { break; };
			container = &*item;
			item_path.push_back(idx);
			items.push((item_path.clone(), item));
		}
		items
	}

	fn remove_item(&mut self, idx: usize) {
		if let Some(items) = self.get_items_mut() {
			items.remove(idx);
		}
	}
}

/// A wish/gift idea on a wishlist.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Item {
	/// The display name of this item.
	pub name: String,
	/// Misc information the user has provided about this item.
	pub description: String,
	/// How many of this item can be reserved.
	pub quantity: usize,
	/// Categories that this item should show up under.
	pub tags: BTreeSet<String>,
	/// The item-subtype for additional properties.
	pub kind: Kind,
}
impl Default for Item {
	fn default() -> Self {
		Self {
			quantity: 1,
			name: Default::default(),
			description: Default::default(),
			tags: Default::default(),
			kind: Default::default(),
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Kind {
	Specific(Specific),
	Idea(Idea),
	Bundle(Bundle),
}
impl Default for Kind {
	fn default() -> Self {
		Kind::Specific(Default::default())
	}
}

/// Item properties which represent a specific gift.
/// These can be purchased directly without any additional considerations from the purchaser.
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Specific {
	pub image_url: Option<String>,
	pub offer_url: String,
	pub cost_per_unit: f32,
	pub cost_per_unit_str: String,
}

/// Item properties which represent a generic idea or concept for a gift.
/// Purchasers will need to find the actual gift themselves.
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Idea {
	pub image_url: Option<String>,
	pub estimated_cost: f32,
	pub estimated_cost_str: String,
	pub example_urls: Vec<String>,
}

/// Item properties which represent a group/set of items.
/// Any item in the bundle can be a specific or idea item, but there shouldn't be interior bundling (this is not enforced).
/// Example: a specific poster and an idea for a frame for the poster.
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Bundle {
	pub entries: Vec<Item>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum KindName {
	Specific,
	Idea,
	Bundle,
}
impl Kind {
	pub fn name(&self) -> KindName {
		match self {
			Kind::Specific(_) => KindName::Specific,
			Kind::Idea(_) => KindName::Idea,
			Kind::Bundle(_) => KindName::Bundle,
		}
	}
}
impl From<KindName> for Kind {
	fn from(name: KindName) -> Self {
		match name {
			KindName::Specific => Self::Specific(Default::default()),
			KindName::Idea => Self::Idea(Default::default()),
			KindName::Bundle => Self::Bundle(Default::default()),
		}
	}
}
impl KindName {
	pub fn all() -> &'static [Self] {
		&[Self::Specific, Self::Idea, Self::Bundle]
	}

	pub fn value(self) -> &'static str {
		match self {
			Self::Specific => "Specific",
			Self::Idea => "Idea",
			Self::Bundle => "Bundle",
		}
	}

	pub fn from(s: &str) -> Self {
		match s {
			"Specific" => Self::Specific,
			"Idea" => Self::Idea,
			"Bundle" => Self::Bundle,
			_ => unimplemented!("No such KindName {s}"),
		}
	}
}
impl FromStr for KindName {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Specific" => Ok(Self::Specific),
			"Idea" => Ok(Self::Idea),
			"Bundle" => Ok(Self::Bundle),
			_ => Err(()),
		}
	}
}

impl Item {
	pub fn set_quantity_from_text(&mut self, text: String) {
		if let Ok(value) = text.parse::<usize>() {
			self.quantity = value.max(1);
		}
	}

	pub fn dec_quantity(&mut self, _: web_sys::MouseEvent) {
		self.quantity = self.quantity.saturating_sub(1).max(1);
	}

	pub fn inc_quantity(&mut self, _: web_sys::MouseEvent) {
		self.quantity = self.quantity.saturating_add(1);
	}
}
impl ItemContainer for Item {
	fn get_items(&self) -> Option<&Vec<Item>> {
		match &self.kind {
			Kind::Bundle(bundle) => Some(&bundle.entries),
			_ => None,
		}
	}

	fn get_items_mut(&mut self) -> Option<&mut Vec<Item>> {
		match &mut self.kind {
			Kind::Bundle(bundle) => Some(&mut bundle.entries),
			_ => None,
		}
	}
}

impl Into<kdl::KdlNode> for Item {
	fn into(self) -> kdl::KdlNode {
		/*
		item "Display Name" {
			description "Long Desc"
			quantity 42
			tags "Home" "D&D" "Kitchen" "Books" "Travel"
			kind "Specific" {
				image_url "https://.../img.png"
				offer_url "https://amazon.com/..."
				cost_per_unit 49.99
			}
		}
		item "Egg Flowers Framed Poster" {
			description "A framed poster"
			quantity 1
			tags "Home"
			kind "Bundle" {
				item "Egg Flowers Poster" {
					description "Egg Flowers Kurzgesagt poster"
					quantity 1
					kind "Specific" {
						image_url "..."
						offer_url "..."
						cost_per_unit 24.90
					}
				}
				item "Small Frame" {
					description "18x24 poster frame"
					quantity 1
					kind "Idea" {
						image_url "..."
						estimated_cost 30
						example_urls {
							"https://amazon.com/..."
						}
					}
				}
			}
		}
		*/
		let mut node = kdl::KdlNode::new("item");
		node.push(self.name);
		node.set_children({
			let mut doc = kdl::KdlDocument::new();
			doc.nodes_mut().push({
				let mut node = kdl::KdlNode::new("description");
				node.push(self.description);
				node
			});
			doc.nodes_mut().push({
				let mut node = kdl::KdlNode::new("quantity");
				node.push(self.quantity as i64);
				node
			});
			if !self.tags.is_empty() {
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("tags");
					for tag in self.tags {
						node.push(tag);
					}
					node
				});
			}
			doc.nodes_mut().push(self.kind.into());
			doc
		});
		node
	}
}

impl Into<kdl::KdlNode> for Kind {
	fn into(self) -> kdl::KdlNode {
		let mut node = kdl::KdlNode::new("kind");
		node.push({
			let mut entry = kdl::KdlEntry::new(self.name().value());
			entry.set_ty("KindName");
			entry
		});
		node.set_children({
			let mut doc = kdl::KdlDocument::new();
			match self {
				Kind::Specific(value) => {
					if let Some(image_url) = value.image_url {
						doc.nodes_mut().push({
							let mut node = kdl::KdlNode::new("image_url");
							node.push(image_url);
							node
						});
					}
					doc.nodes_mut().push({
						let mut node = kdl::KdlNode::new("offer_url");
						node.push(value.offer_url);
						node
					});
					doc.nodes_mut().push({
						let mut node = kdl::KdlNode::new("cost_per_unit");
						node.push(value.cost_per_unit as f64);
						node
					});
				}
				Kind::Idea(value) => {
					if let Some(image_url) = value.image_url {
						doc.nodes_mut().push({
							let mut node = kdl::KdlNode::new("image_url");
							node.push(image_url);
							node
						});
					}
					doc.nodes_mut().push({
						let mut node = kdl::KdlNode::new("estimated_cost");
						node.push(value.estimated_cost as f64);
						node
					});
					if !value.example_urls.is_empty() {
						doc.nodes_mut().push({
							let mut node = kdl::KdlNode::new("example_urls");
							node.set_children({
								let mut doc = kdl::KdlDocument::new();
								for url in value.example_urls {
									doc.nodes_mut().push(kdl::KdlNode::new(url));
								}
								doc
							});
							node
						});
					}
				}
				Kind::Bundle(value) => {
					for item in value.entries {
						doc.nodes_mut().push(item.into());
					}
				}
			}
			doc
		});
		node
	}
}

impl TryFrom<&kdl::KdlNode> for Item {
	type Error = anyhow::Error;

	fn try_from(node: &kdl::KdlNode) -> Result<Self, Self::Error> {
		use crate::api::github::gist::KdlParseError::*;
		let name = {
			let entry = node.get(0).ok_or(NoArgument("name", 0))?;
			entry
				.value()
				.as_string()
				.ok_or(InvalidArgValue("name", 0, "string"))?
		}
		.to_owned();
		let children = node
			.children()
			.ok_or(NoChildrenOf(node.name().value().into()))?;
		let description = {
			let node = children
				.get("description")
				.ok_or(NoNode("item::description"))?;
			let entry = node.get(0).ok_or(NoArgument("item::description", 0))?;
			entry
				.value()
				.as_string()
				.ok_or(InvalidArgValue("item::description", 0, "string"))?
		}
		.to_owned();
		let quantity = {
			let node = children.get("quantity").ok_or(NoNode("item::quantity"))?;
			let entry = node.get(0).ok_or(NoArgument("item::quantity", 0))?;
			entry
				.value()
				.as_i64()
				.ok_or(InvalidArgValue("item::quantity", 0, "i64"))?
		} as usize;
		let tags = match children.get("tags") {
			Some(node) => node
				.entries()
				.iter()
				.filter_map(|entry| entry.value().as_string())
				.map(str::to_owned)
				.collect(),
			None => Default::default(),
		};
		let kind = {
			let node = children.get("kind").ok_or(NoNode("item::kind"))?;
			Kind::try_from(node)?
		};
		Ok(Self {
			name,
			description,
			quantity,
			tags,
			kind,
		})
	}
}

impl TryFrom<&kdl::KdlNode> for Kind {
	type Error = anyhow::Error;

	fn try_from(node: &kdl::KdlNode) -> Result<Self, Self::Error> {
		use crate::api::github::gist::KdlParseError::*;
		let entry = node.get(0).ok_or(NoArgument("kind", 0))?;
		let kind_name_str = entry
			.value()
			.as_string()
			.ok_or(InvalidArgValue("kind", 0, "string"))?;
		let kind_name = KindName::from_str(kind_name_str)
			.map_err(|_| InvalidArgValue("kind", 0, "wishlist::KindName"))?;
		let doc = node.children().ok_or(NoChildrenOf("kind".into()))?;
		Ok(match kind_name {
			KindName::Specific => {
				let image_url = match doc.get("image_url") {
					Some(node) => {
						let entry = node.get(0).ok_or(NoArgument("kind::image_url", 0))?;
						let value = entry.value().as_string().ok_or(InvalidArgValue(
							"kind::image_url",
							0,
							"string",
						))?;
						Some(value.to_owned())
					}
					None => None,
				};
				let offer_url = {
					let node = doc.get("offer_url").ok_or(NoNode("kind::offer_url"))?;
					let entry = node.get(0).ok_or(NoArgument("kind::offer_url", 0))?;
					let value = entry.value().as_string().ok_or(InvalidArgValue(
						"kind::offer_url",
						0,
						"string",
					))?;
					value.to_owned()
				};
				let cost_per_unit = {
					let node = doc
						.get("cost_per_unit")
						.ok_or(NoNode("kind::cost_per_unit"))?;
					let entry = node.get(0).ok_or(NoArgument("kind::cost_per_unit", 0))?;
					entry.value().as_f64().ok_or(InvalidArgValue(
						"kind::cost_per_unit",
						0,
						"f64",
					))?
				} as f32;
				Self::Specific(Specific {
					image_url,
					offer_url,
					cost_per_unit,
					cost_per_unit_str: format!("{cost_per_unit:.2}"),
				})
			}
			KindName::Idea => {
				let image_url = match doc.get("image_url") {
					Some(node) => {
						let entry = node.get(0).ok_or(NoArgument("kind::image_url", 0))?;
						let value = entry.value().as_string().ok_or(InvalidArgValue(
							"kind::image_url",
							0,
							"string",
						))?;
						Some(value.to_owned())
					}
					None => None,
				};
				let estimated_cost = {
					let node = doc
						.get("estimated_cost")
						.ok_or(NoNode("kind::estimated_cost"))?;
					let entry = node.get(0).ok_or(NoArgument("kind::estimated_cost", 0))?;
					entry.value().as_f64().ok_or(InvalidArgValue(
						"kind::estimated_cost",
						0,
						"f64",
					))?
				} as f32;
				let example_urls = match doc.get("example_urls") {
					Some(node) => {
						let doc = node
							.children()
							.ok_or(NoChildrenOf("kind::example_urls".into()))?;
						doc.nodes()
							.iter()
							.map(|node| node.name().value().to_owned())
							.collect()
					}
					None => Vec::new(),
				};
				Self::Idea(Idea {
					image_url,
					estimated_cost,
					estimated_cost_str: format!("{estimated_cost:.2}"),
					example_urls,
				})
			}
			KindName::Bundle => {
				let mut entries = Vec::new();
				for node in doc.nodes() {
					entries.push(Item::try_from(node)?);
				}
				Self::Bundle(Bundle { entries })
			}
		})
	}
}
