use std::{collections::BTreeMap, rc::Rc};
use yew::{
	prelude::*,
	suspense::{SuspensionResult, UseFutureHandle},
};

pub struct UseQueryHandle<Args, Output, Error> {
	handle: UseFutureHandle<Result<Output, Error>>,
	run: Rc<dyn Fn(Args)>,
}
impl<Args, Output, Error> UseQueryHandle<Args, Output, Error> {
	pub fn get_trigger(&self) -> &Rc<dyn Fn(Args)> {
		&self.run
	}

	pub fn status(&self) -> &Result<Output, Error> {
		&*self.handle
	}
}

#[hook]
pub fn use_query_all<D>() -> SuspensionResult<UseQueryHandle<(), BTreeMap<String, (D::Record, D)>, D::Error>>
where
	D: crate::data::RecordData + Clone + 'static + std::fmt::Debug,
	D::Record: database::Record + for<'de> serde::Deserialize<'de> + Unpin + Clone + 'static + std::fmt::Debug,
	D::Error: From<database::Error> + Clone + 'static,
{
	use database::{ObjectStoreExt, Record, TransactionExt};
	use futures_util::StreamExt;
	let database = use_context::<super::Database>().unwrap();
	let recycle = use_state(|| false);
	let handle = yew::suspense::use_future_with(recycle.clone(), |_recycle| async move {
		let transaction = database.read()?;
		let store = transaction.object_store_of::<D::Record>()?;
		let mut cursor = store.cursor_all::<D::Record>().await?;
		let mut parsed_data = BTreeMap::new();
		while let Some(record) = cursor.next().await {
			let Some(key) = record.key() else { continue; };
			let data = D::parse_record(&record)?;
			parsed_data.insert(key.clone(), (record, data));
		}
		Ok(parsed_data)
	})?;
	let run = Rc::new({
		let recycle = recycle.clone();
		move |_args: ()| {
			recycle.set(!*recycle);
		}
	});
	Ok(UseQueryHandle { handle, run })
}

#[hook]
pub fn use_query_discrete<D>(
	key: String,
) -> SuspensionResult<UseQueryHandle<Option<String>, Option<(D::Record, D)>, D::Error>>
where
	D: crate::data::RecordData + Clone + 'static + std::fmt::Debug,
	D::Record: database::Record + for<'de> serde::Deserialize<'de> + Unpin + Clone + 'static + std::fmt::Debug,
	D::Error: From<database::Error> + Clone + 'static,
{
	let database = use_context::<super::Database>().unwrap();
	let args = use_state(move || (key, false));
	let handle = yew::suspense::use_future_with(args.clone(), |args| async move {
		let key = &(*args).0;

		let Some(record) = database.get::<D::Record>(key).await? else {
				return Ok(None);
			};
		let data = D::parse_record(&record)?;
		Ok(Some((record, data)))
	})?;
	let run = Rc::new({
		let args_state = args.clone();
		move |key: Option<String>| {
			let recycle = !(*args_state).1;
			if let Some(key) = key {
				args_state.set((key, recycle));
			} else {
				args_state.set(((*args_state).0.clone(), recycle));
			}
		}
	});
	Ok(UseQueryHandle { handle, run })
}
