use super::{Bundle, Idea, Specific};
use crate::util::error::InvalidEnumStr;
use enumset::EnumSetType;
use kdlize::{error::UserProvidedError, AsKdl, FromKdl};
use std::str::FromStr;

#[derive(EnumSetType, Debug)]
pub enum KindId {
	Specific,
	Idea,
	Bundle,
}
impl ToString for KindId {
	fn to_string(&self) -> String {
		match self {
			Self::Specific => "Specific",
			Self::Idea => "Idea",
			Self::Bundle => "Bundle",
		}
		.to_owned()
	}
}
impl FromStr for KindId {
	type Err = kdlize::error::UserProvidedError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Specific" => Ok(Self::Specific),
			"Idea" => Ok(Self::Idea),
			"Bundle" => Ok(Self::Bundle),
			_ => Err(UserProvidedError::from_error(InvalidEnumStr::<Self>::from(s))),
		}
	}
}
impl KindId {
	pub fn help_info(&self) -> &'static str {
		match self {
			Self::Specific => "A specific offer found at the provided url. If gifting this item, please purchase the one at the url provided below.",
			Self::Idea => "An idea for a gift. Example offers may be provided, but please use you're best judgement when purchasing an item. The item need not be one listed below, just something like these.",
			Self::Bundle => "A set of items. If purchasing this, please purchase all of its contents together. Example: a picture (idea) and a specific frame.",
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
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

impl Kind {
	pub fn id(&self) -> KindId {
		match self {
			Self::Specific(_) => KindId::Specific,
			Self::Idea(_) => KindId::Idea,
			Self::Bundle(_) => KindId::Bundle,
		}
	}

	pub fn image_urls(&self) -> Vec<&String> {
		let mut urls = Vec::with_capacity(1);
		match self {
			Self::Specific(specific) => {
				if let Some(url) = &specific.image_url {
					urls.push(url);
				}
			}
			Self::Idea(idea) => {
				if let Some(url) = &idea.image_url {
					urls.push(url);
				}
			}
			Self::Bundle(bundle) => {
				for entry in &bundle.entries {
					urls.extend(entry.kind.image_urls());
				}
			}
		}
		urls
	}
}
impl From<KindId> for Kind {
	fn from(name: KindId) -> Self {
		match name {
			KindId::Specific => Self::Specific(Default::default()),
			KindId::Idea => Self::Idea(Default::default()),
			KindId::Bundle => Self::Bundle(Default::default()),
		}
	}
}

impl FromKdl<()> for Kind {
	type Error = anyhow::Error;

	fn from_kdl<'doc>(node: &mut kdlize::NodeReader<'doc, ()>) -> Result<Self, Self::Error> {
		match node.next_str_req_t()? {
			KindId::Specific => Ok(Self::Specific(Specific::from_kdl(node)?)),
			KindId::Idea => Ok(Self::Idea(Idea::from_kdl(node)?)),
			KindId::Bundle => Ok(Self::Bundle(Bundle::from_kdl(node)?)),
		}
	}
}

impl AsKdl for Kind {
	fn as_kdl(&self) -> kdlize::NodeBuilder {
		let mut node = kdlize::NodeBuilder::default();
		node.push_entry(self.id().to_string());
		node += match self {
			Self::Specific(value) => value.as_kdl(),
			Self::Idea(value) => value.as_kdl(),
			Self::Bundle(value) => value.as_kdl(),
		};
		node
	}
}
