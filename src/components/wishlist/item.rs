use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
}

/// Item properties which represent a generic idea or concept for a gift.
/// Purchasers will need to find the actual gift themselves.
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Idea {
	pub image_url: Option<String>,
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

impl Item {
	pub fn set_quantity_from_text((item, text): (&mut Self, String)) {
		if let Ok(value) = text.parse::<usize>() {
			item.quantity = value.max(1);
		}
	}

	pub fn dec_quantity((item, _): (&mut Self, web_sys::MouseEvent)) {
		item.quantity = item.quantity.saturating_sub(1).max(1);
	}

	pub fn inc_quantity((item, _): (&mut Self, web_sys::MouseEvent)) {
		item.quantity = item.quantity.saturating_add(1);
	}
}
