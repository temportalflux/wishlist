use yew::prelude::*;

mod schema;
use schema::*;
mod list;
pub use list::*;
mod user;
pub use user::*;

#[derive(Clone, PartialEq)]
pub struct Database(database::Client);

impl Database {
	pub async fn open() -> Result<Self, database::Error> {
		Ok(Self(database::Client::open::<SchemaVersion>("wishlist").await?))
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
