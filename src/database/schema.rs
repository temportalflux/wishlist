use super::{List, User};
use database::{Client, Error, MissingVersion, Record, Schema, Transaction};

pub enum SchemaVersion {
	Version1 = 1,
}

impl TryFrom<u32> for SchemaVersion {
	type Error = MissingVersion;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(Self::Version1),
			_ => Err(MissingVersion(value)),
		}
	}
}

impl Schema for SchemaVersion {
	fn latest() -> u32 {
		Self::Version1 as u32
	}

	fn apply(&self, database: &Client, _transaction: Option<&Transaction>) -> Result<(), Error> {
		match self {
			Self::Version1 => {
				// Create lists table
				{
					let mut params = idb::ObjectStoreParams::new();
					params.auto_increment(true);
					params.key_path(Some(idb::KeyPath::new_single("id")));
					let _store = database.create_object_store(List::store_id(), params)?;
				}
				// Create users table
				{
					let mut params = idb::ObjectStoreParams::new();
					params.auto_increment(true);
					params.key_path(Some(idb::KeyPath::new_single("login")));
					let _store = database.create_object_store(User::store_id(), params)?;
				}
			}
		}
		Ok(())
	}
}
