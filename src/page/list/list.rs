use crate::{
	data::List as ListData,
	database::{
		query::{use_query_discrete, QueryStatus},
		ListId,
	},
	spinner::Spinner,
	GeneralProp,
};
use yew::prelude::*;

#[function_component]
pub fn List(GeneralProp { value }: &GeneralProp<ListId>) -> Html {
	let list = use_query_discrete::<ListData>(value.to_string(), true);
	match list.status() {
		QueryStatus::Pending => html!(<Spinner />),
		QueryStatus::Empty | QueryStatus::Success(None) => {
			html!(format!("No such list found for id {:?}", value.to_string()))
		}
		QueryStatus::Failed(err) => html!(format!("Failed to parse list: {err:?}")),
		QueryStatus::Success(Some((record, data))) => {
			html!(format!("{record:?}\n{data:?}"))
		}
	}
}
