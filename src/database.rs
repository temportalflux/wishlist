use database::Record;
use futures_util::future::LocalBoxFuture;
use yew::prelude::*;

mod schema;
use schema::*;
mod list;
pub use list::*;
mod user;
pub use user::*;
pub mod query;

#[derive(Clone, PartialEq)]
pub struct Database(database::Client);

impl Database {
	pub async fn open() -> Result<Self, database::Error> {
		Ok(Self(database::Client::open::<SchemaVersion>("wishlist").await?))
	}

	pub fn write(&self) -> Result<database::Transaction, database::Error> {
		self.0
			.transaction(&[User::store_id(), List::store_id()], idb::TransactionMode::ReadWrite)
	}

	pub fn read(&self) -> Result<database::Transaction, database::Error> {
		self.0
			.transaction(&[User::store_id(), List::store_id()], idb::TransactionMode::ReadOnly)
	}

	pub async fn get<T>(&self, key: impl Into<wasm_bindgen::JsValue>) -> Result<Option<T>, database::Error>
	where
		T: Record + serde::de::DeserializeOwned,
	{
		self.0.get::<T>(key).await
	}

	pub async fn mutate<F>(&self, fn_transaction: F) -> Result<(), database::Error>
	where
		F: FnOnce(&database::Transaction) -> LocalBoxFuture<'_, Result<(), database::Error>>,
	{
		let transaction = self.write()?;
		fn_transaction(&transaction).await?;
		transaction.commit().await?;
		Ok(())
	}

	pub async fn new_list_id(&self, owner: &str) -> Result<ListId, database::Error> {
		let mut list_id = ListId {
			owner: owner.to_owned(),
			id: String::default(),
		};
		let mut found_list = true;
		while found_list {
			list_id.id = nanoid::nanoid!(10, &nanoid::alphabet::SAFE).to_owned();
			found_list = self.get::<List>(list_id.to_string()).await?.is_some();
		}
		Ok(list_id)
	}
}

#[function_component]
pub fn Provider(props: &html::ChildrenProps) -> Html {
	let database = yew_hooks::use_async(async move {
		match Database::open().await {
			Ok(db) => Ok(db),
			Err(err) => {
				log::error!(target: env!("CARGO_PKG_NAME"), "Failed to connect to database: {err:?}");
				Err(std::sync::Arc::new(err))
			}
		}
	});
	// When the app first opens, load the database.
	// Could probably check `use_is_first_mount()`, but checking if there database
	// doesn't exist yet and isn't loading is more clear.
	if database.data.is_none() && !database.loading {
		log::info!(target: env!("CARGO_PKG_NAME"), "Initializing database");
		database.run();
	}
	// If the database has not yet loaded (or encountered an error),
	// we wont even show the children - mostly to avoid the numerous errors that would occur
	// since children strongly rely on the database existing.
	let Some(ddb) = &database.data else {
		return html!();
	};
	html! {
		<ContextProvider<Database> context={ddb.clone()}>
			{props.children.clone()}
		</ContextProvider<Database>>
	}
}
