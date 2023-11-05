use crate::{
	Route,
	components::{Tag, Tags},
	data::{Entry, Kind, List as ListData},
	database::{query::use_query_discrete, List as ListRecord, ListId},
};
use std::{collections::BTreeSet, rc::Rc};
use itertools::{Itertools, Position};
use yew::prelude::*;
use yew_router::prelude::{Link, use_navigator};
use yewdux::prelude::use_store_value;

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

#[derive(Clone, PartialEq, Properties)]
pub struct ListProps {
	pub list_id: ListId,
	pub entry_path: Option<EntryPath>,
}
#[function_component]
pub fn List(ListProps { list_id, entry_path }: &ListProps) -> HtmlResult {
	let query = use_query_discrete::<ListData>(list_id.to_string())?;
	Ok(match query.status() {
		Err(err) => html!(format!("Failed to parse list: {err:?}")),
		Ok(None) => html!(format!("404 no such list found")),
		Ok(Some((record, data))) => {
			html!(<ListBody
				list_id={list_id.clone()}
				record={record.clone()} data={data.clone()}
				entry_path={*entry_path}
			/>)
		}
	})
}

#[derive(Clone, PartialEq, Properties)]
pub struct ListBodyProps {
	pub list_id: ListId,
	pub record: ListRecord,
	pub data: ListData,
	pub entry_path: Option<EntryPath>,
}
#[function_component]
fn ListBody(ListBodyProps { list_id, record, data, entry_path }: &ListBodyProps) -> Html {
	let list = use_reducer({
		let editable = EditableList {
			record: Rc::new(record.clone()),
			data: data.clone(),
		};
		move || editable
	});

	let entry_at_path = match entry_path {
		None => None,
		Some(path) => match path.resolve_from(&list.data) {
			None => None,
			Some((crumbs, entry)) => Some((path, crumbs, entry)),
		},
	};
	let unique_tags = list.data.entries.iter().fold(BTreeSet::default(), |mut tags, entry| {
		tags.extend(entry.tags.iter().map(|s| AttrValue::from(s.clone())));
		tags
	});

	match entry_at_path {
		None => {
			html!(<ListContent list_id={list_id.clone()} list={list} {unique_tags} />)
		}
		Some((path, crumbs, entry)) => html! {
			<div>
				<nav style="--bs-breadcrumb-divider: '>';" aria-label="breadcrumb">
					<ol class="breadcrumb">
						{crumbs.into_iter().with_position().map(|(position, (name, path))| {
							let mut classes = classes!("breadcrumb-item");
							match position {
								Position::Last | Position::Only => {
									classes.push("active");
								}
								_ => {}
							}
							html! {
								<li class={classes}>
									<Link<Route> to={Route::new_list_entry(list_id.clone(), path)}>
										{name}
									</Link<Route>>
								</li>
							}
						}).collect::<Vec<_>>()}
					</ol>
				</nav>
				<EntryContent list={list.clone()} path={*path} {unique_tags} />
			</div>
		},
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ListContentProps {
	list_id: ListId,
	list: UseReducerHandle<EditableList>,
	unique_tags: BTreeSet<AttrValue>,
}
#[function_component]
fn ListContent(props: &ListContentProps) -> Html {
	let ListContentProps { list_id, list, unique_tags } = props;
	let navigator = use_navigator().unwrap();

	// TODO-QoL: maybe save to session storage?
	let tag_filter = use_state_eq(|| BTreeSet::default());

	let save_to_database = Callback::from(|_message: String| {});
	let add_entry = Callback::from({
		let list_id = list_id.clone();
		let list = list.clone();
		let navigator = navigator.clone();
		let save_to_database = save_to_database.clone();
		move |_| {
			let route = Route::new_list_entry(list_id.clone(), Some(EntryPath::root(0)));
			let navigator = navigator.clone();
			let save_to_database = save_to_database.clone();
			list.dispatch(Box::new(move |data: &mut ListData| {
				let mut entry = crate::data::Entry::default();
				entry.name = format!("CustomItem");
				data.entries.insert(0, entry);
				save_to_database.emit(format!("Add an item"));
				navigator.push(&route);
			}));
		}
	});
	let delete_entry = Callback::from({
		let list = list.clone();
		let save_to_database = save_to_database.clone();
		move |path: EntryPath| {
			let save_to_database = save_to_database.clone();
			list.dispatch(Box::new(move |data: &mut ListData| {
				match path.bundle_idx {
					None => {
						if data.entries.get(path.root).is_some() {
							data.entries.remove(path.root);
							save_to_database.emit(format!("Delete an item"));
						}
					}
					Some(bundle_idx) => {
						let Some(entry) = data.entries.get_mut(path.root) else {
							return;
						};
						let Kind::Bundle(bundle) = &mut entry.kind else {
							return;
						};
						if bundle.entries.get(bundle_idx).is_some() {
							bundle.entries.remove(bundle_idx);
							save_to_database.emit(format!("Delete an item in a bundle"));
						}
					}
				}
			}))
		}
	});

	let invite_user = Callback::from({
		let list = list.clone();
		let save_to_database = save_to_database.clone();
		move |_| {}
	});
	let remove_user = Callback::from({
		let list = list.clone();
		let save_to_database = save_to_database.clone();
		move |_user_id: AttrValue| {}
	});

	let toggle_tag = Callback::from({
		let tag_filter = tag_filter.clone();
		move |tag: AttrValue| {
			if tag_filter.contains(&tag) {
				let mut filter = (*tag_filter).clone();
				filter.remove(&tag);
				tag_filter.set(filter);
			}
			else {
				let mut filter = (*tag_filter).clone();
				filter.insert(tag);
				tag_filter.set(filter);
			}
		}
	});

	html! {
		<div class="list page">
			<h2>{&list.data.name}</h2>
			<div>{format!("Invited Users: {:?}", list.data.invitees)}</div>

			<div class="d-flex">
				<button class="btn btn-success btn-sm me-3" onclick={invite_user}>
					<i class="bi bi-plus" />
					{"Invite Giver"}
				</button>
				<Tags>
					{list.data.invitees.iter().map(|user_id| {
						let user_id: AttrValue = user_id.clone().into();
						let onclick = remove_user.reform({
							let user_id = user_id.clone();
							move |_| user_id.clone()
						});
						html!(<Tag classes={"is-rounded"}>
							{user_id.clone()}
							<i class="bi bi-x-circle" {onclick} />
						</Tag>)
					}).collect::<Vec<_>>()}
				</Tags>
			</div>

			<Tags>
				{unique_tags.iter().map(|tag| {
					let on_click = toggle_tag.reform({
						let tag = tag.clone();
						move |_| tag.clone()
					});
					html!(<Tag classes="is-rounded" active={tag_filter.contains(tag)} {on_click}>
						{tag.clone()}
					</Tag>)
				}).collect::<Vec<_>>()}
			</Tags>

			<div class="d-flex flex-wrap">
				<div class="card entry m-2">
					<div class="card-body d-flex align-items-center justify-content-center">
						<button class="btn btn-success" onclick={add_entry}>
							<i class="bi bi-plus" />
							{"Add Item"}
						</button>
					</div>
				</div>
				{list.data.entries.iter().enumerate().map(|(idx, entry)| {
					let path = EntryPath::root(idx);
					let route = Route::new_list_entry(list_id.clone(), Some(path));
					let delete_entry = delete_entry.reform(move |_| path);
					entry_card(entry, route, delete_entry)
				}).collect::<Vec<_>>()}
			</div>
		</div>
	}
}

#[derive(Clone, Copy, PartialEq)]
pub struct EntryPath {
	pub root: usize,
	pub bundle_idx: Option<usize>,
}
impl EntryPath {
	pub fn root(idx: usize) -> Self {
		Self {
			root: idx,
			bundle_idx: None,
		}
	}

	pub fn bundled(&self, idx: usize) -> Self {
		Self {
			root: self.root,
			bundle_idx: Some(idx),
		}
	}

	fn resolve_from<'list>(&self, list: &'list ListData) -> Option<(Vec<(&'list String, Option<EntryPath>)>, &'list Entry)> {
		let mut crumbs = Vec::with_capacity(3);
		crumbs.push((&list.name, None));
		let Some(entry) = list.entries.get(self.root) else {
			return None;
		};
		crumbs.push((&entry.name, Some(Self::root(self.root))));
		let Some(bundle_idx) = self.bundle_idx else {
			return Some((crumbs, entry));
		};
		let Kind::Bundle(bundle) = &entry.kind else {
			return None;
		};
		let Some(entry) = bundle.entries.get(bundle_idx) else {
			return None;
		};
		crumbs.push((&entry.name, Some(*self)));
		Some((crumbs, entry))
	}
}

fn entry_card(entry: &Entry, route: Route, delete: Callback<()>) -> Html {
	let kind_id = entry.kind.id().to_string();

	let image_urls = entry.kind.image_urls();
	let image = match image_urls.len() {
		0 => html!(),
		1 => html!(<img class="card-img-top" src={image_urls.into_iter().next().cloned()} />),
		_ => html!(<div>{image_urls.into_iter().map(|url| {
			html!(<img src={url.clone()} />)
		}).collect::<Vec<_>>()}</div>),
	};
	let view_at_url = match &entry.kind {
		Kind::Specific(specific) if !specific.offer_url.is_empty() => html! {
			<a class="icon-link" target="_blank" href={specific.offer_url.clone()}>
				{"View Url"}
				<i class="bi bi-box-arrow-up-right" style="height: inheiret;" />
			</a>
		},
		_ => html!(),
	};

	html! {
		<div class={classes!("card", "entry", "m-2", kind_id.to_lowercase())}>
			<div class="card-header d-flex align-items-center">
				<span>{&kind_id}</span>
				{match entry.quantity {
					0 | 1 => html!(),
					n => html!(<span>{n - entry.reservations.len()}{" / "}{n}{" remaining"}</span>)
				}}
				<i
					class={"bi bi-trash ms-auto ps-3"}
					onclick={delete.reform(|_| ())}
				/>
			</div>
			{image}
			<div class="card-body">
				<h5 class="card-title">{&entry.name}</h5>
				<div>{&entry.description}</div>
			</div>
			<div class="card-footer d-flex">
				{view_at_url}
				<Link<Route> classes="icon-link icon-link-hover ms-auto" to={route}>
					{"Open"}
					<i class="bi bi-chevron-right" style="height: inheiret;" />
				</Link<Route>>
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct EntryContentProps {
	list: UseReducerHandle<EditableList>,
	path: EntryPath,
	unique_tags: BTreeSet<AttrValue>,
}
#[function_component]
fn EntryContent(EntryContentProps { list, path, unique_tags }: &EntryContentProps) -> Html {
	let auth_info = use_store_value::<crate::auth::Info>();
	let Some((_crumbs, entry)) = path.resolve_from(&list.data) else {
		return html!("404 entry not found - todo better display");
	};
	let is_owner = list.record.id.starts_with(&auth_info.name);
	let form_control = match is_owner {
		true => "form-control",
		false => "form-control-plaintext",
	};

	// TODO: quantity, tags, and kind (below)
	// TODO: reserve btn, delete btn
	// TODO: specific kind; image url, offer url, cost
	// TODO: idea kind; image url, estimated cost, example urls
	// TODO: bundle kind; entry cards/editor

	html! {<div class="entry page">
		<div class="mb-3">
			<label for="name" class="form-label">{"Title"}</label>
			<input
				type="text"
				class={form_control} readonly={!is_owner}
				id="name" value={entry.name.clone()}
			/>
		</div>
		<div class="mb-3">
			<label class="form-label" for="description">{"Description"}</label>
			<textarea rows="3"
				class={form_control} readonly={!is_owner}
				id="description"
			>
				{&entry.description}
			</textarea>
		</div>
	</div>}
}
