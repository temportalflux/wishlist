mod list;
pub use list::*;

mod user;
pub use user::*;

pub trait RecordData {
	type Record: database::Record;
	type Error;
	fn parse_record(record: &Self::Record) -> Result<Self, Self::Error>
	where
		Self: Sized;
}
