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
	type Error = kdlize::error::Error;

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
