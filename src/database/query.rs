use std::collections::BTreeMap;

use yew::prelude::*;
use yew_hooks::UseAsyncHandle;

#[derive(Debug, PartialEq)]
pub enum QueryStatus<T, E> {
	Empty,
	Pending,
	Success(T),
	Failed(E),
}

#[derive(Clone)]
pub struct UseQueryHandle<Args, Output, Error> {
	async_handle: UseAsyncHandle<Output, Error>,
	run: std::rc::Rc<dyn Fn(Args)>,
}
impl<Args, Output, Error> PartialEq for UseQueryHandle<Args, Output, Error>
where
	Output: PartialEq,
	Error: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.async_handle == other.async_handle
	}
}
impl<Args, Output, Error> UseQueryHandle<Args, Output, Error> {
	pub fn status(&self) -> QueryStatus<&Output, &Error> {
		if self.async_handle.loading {
			return QueryStatus::Pending;
		}
		if let Some(error) = &self.async_handle.error {
			return QueryStatus::Failed(error);
		}
		if let Some(data) = &self.async_handle.data {
			return QueryStatus::Success(data);
		}
		QueryStatus::Empty
	}

	pub fn run(&self, args: Args) {
		(self.run)(args);
	}
}

#[hook]
pub fn use_query_all<D>(auto_fetch: bool) -> UseQueryHandle<(), BTreeMap<String, (D::Record, D)>, D::Error>
where
	D: crate::data::RecordData + Clone + 'static + std::fmt::Debug,
	D::Record: database::Record + for<'de> serde::Deserialize<'de> + Unpin + Clone + 'static + std::fmt::Debug,
	D::Error: From<database::Error> + Clone + 'static,
{
	use database::{ObjectStoreExt, Record, TransactionExt};
	use futures_util::StreamExt;
	let database = use_context::<super::Database>().unwrap();
	let options = yew_hooks::UseAsyncOptions { auto: auto_fetch };
	let async_handle = yew_hooks::use_async_with_options(
		async move {
			let transaction = database.read()?;
			let store = transaction.object_store_of::<D::Record>()?;
			let mut cursor = store.cursor_all::<D::Record>().await?;
			let mut parsed_data = BTreeMap::new();
			while let Some(record) = cursor.next().await {
				let Some(key) = record.key() else { continue; };
				let data = D::parse_record(&record)?;
				parsed_data.insert(key.clone(), (record, data));
			}
			log::debug!("{parsed_data:?}");
			Ok(parsed_data)
		},
		options,
	);
	let run = std::rc::Rc::new({
		let handle = async_handle.clone();
		move |_args: ()| {
			handle.run();
		}
	});
	UseQueryHandle { async_handle, run }
}

#[hook]
pub fn use_query_discrete<D>(
	key: String,
	auto_fetch: bool,
) -> UseQueryHandle<Option<String>, Option<(D::Record, D)>, D::Error>
where
	D: crate::data::RecordData + Clone + 'static + std::fmt::Debug,
	D::Record: database::Record + for<'de> serde::Deserialize<'de> + Unpin + Clone + 'static + std::fmt::Debug,
	D::Error: From<database::Error> + Clone + 'static,
{
	let database = use_context::<super::Database>().unwrap();
	let options = yew_hooks::UseAsyncOptions { auto: auto_fetch };
	let args_handle = std::rc::Rc::new(std::sync::Mutex::new(key));
	let async_args = args_handle.clone();
	let async_handle = yew_hooks::use_async_with_options(
		async move {
			let guard = async_args.lock().unwrap();
			let key = &*guard;

			let Some(record) = database.get::<D::Record>(key).await? else {
				return Ok(None);
			};
			let data = D::parse_record(&record)?;
			Ok(Some((record, data)))
		},
		options,
	);
	let run = std::rc::Rc::new({
		let handle = async_handle.clone();
		move |args: Option<String>| {
			if let Some(args) = args {
				*args_handle.lock().unwrap() = args;
			}
			handle.run();
		}
	});
	UseQueryHandle { async_handle, run }
}
