use crate::{
	components::Spinner,
	data::{List as ListData, RecordError},
	database::{
		query::{use_query_discrete, QueryStatus},
		List as ListRecord, ListId,
	},
	GeneralProp,
};
use std::rc::Rc;
use yew::{prelude::*, suspense::SuspensionResult};

#[derive(Clone, PartialEq)]
struct EditableList {
	record: Rc<ListRecord>,
	data: ListData,
}
impl Default for EditableList {
	fn default() -> Self {
		Self {
			record: Rc::new(ListRecord::default()),
			data: ListData::default(),
		}
	}
}
impl Reducible for EditableList {
	type Action = Box<dyn FnOnce(&mut ListData) + 'static>;

	fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
		let mut data = self.data.clone();
		action(&mut data);
		match data != self.data {
			true => Rc::new(Self {
				record: self.record.clone(),
				data,
			}),
			false => self,
		}
	}
}

#[function_component]
pub fn List(GeneralProp { value }: &GeneralProp<ListId>) -> HtmlResult {
	let query = use_query_discrete::<ListData>(value.to_string())?;
	Ok(match query.status() {
		Err(err) => html!(format!("Failed to parse list: {err:?}")),
		Ok(None) => html!(format!("404 no such list found")),
		Ok(Some(query_data)) => {
			html!(<ListBody value={query_data.clone()} />)
		}
	})
}

#[function_component]
fn ListBody(GeneralProp { value: (record, data) }: &GeneralProp<(ListRecord, ListData)>) -> Html {
	let list = use_reducer({
		let editable = EditableList {
			record: Rc::new(record.clone()),
			data: data.clone(),
		};
		move || editable
	});

	let save_to_database = Callback::from(|_: ()| {});
	let add_item = Callback::from({
		let list = list.clone();
		move |_| {
			list.dispatch(Box::new(move |data: &mut ListData| {
				data.entries.insert(0, crate::data::Entry::default());
			}));
		}
	});
	let delete_entry = Callback::from({
		let list = list.clone();
		move |idx| {
			list.dispatch(Box::new(move |data: &mut ListData| {
				data.entries.remove(idx);
			}))
		}
	});

	html! {
		<div class="list page">
			<h2>{&list.data.name}</h2>
			<div>{format!("{:?}", list.data.invitees)}</div>
			<button class="btn btn-success" onclick={add_item}>
				<i class="bi bi-plus" />
				{"Add Item"}
			</button>
			<div class="d-flex flex-wrap">
				{list.data.entries.iter().enumerate().map(|(idx, entry)| {
					html! {
						<div class="card entry m-2">
							<div class="card-header d-flex align-items-center">
								<span>{&entry.name}</span>
								<i
									class={"bi bi-trash ms-auto ps-3"}
									onclick={delete_entry.reform(move |_| idx)}
								/>
							</div>
							<div class="card-body">
								{format!("{entry:?}")}
							</div>
						</div>
					}
				}).collect::<Vec<_>>()}
			</div>
		</div>
	}
}
