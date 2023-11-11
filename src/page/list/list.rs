use crate::{
	components::{Tag, Tags},
	data::{Entry, Kind, KindId, List as ListData},
	database::{query::use_query_discrete, Database, List as ListRecord, ListId},
	util::web_ext::{validate_uint_only, CallbackExt, CallbackOptExt, InputExt},
	Route,
};
use derivative::Derivative;
use enumset::EnumSet;
use itertools::{Itertools, Position};
use kdlize::{ext::NodeExt, AsKdl, NodeId};
use std::{
	collections::{BTreeMap, BTreeSet},
	rc::Rc,
	str::FromStr,
	sync::atomic::AtomicBool,
};
use yew::prelude::*;
use yew_router::prelude::{use_navigator, Link};
use yewdux::prelude::use_store_value;

#[derive(Clone, Derivative)]
#[derivative(PartialEq)]
struct EditableList {
	id: Rc<ListId>,
	record: Rc<ListRecord>,
	data: ListData,
	#[derivative(PartialEq = "ignore")]
	database: Database,
	#[derivative(PartialEq = "ignore")]
	save_to_storage_timeout: Option<Rc<AtomicBool>>,
}
enum EditableAction {
	MutateData(Box<dyn FnOnce(&mut ListData) + 'static>),
	SavedToDatabase {
		record: Rc<ListRecord>,
		save_to_storage_timeout: Rc<AtomicBool>,
	},
	SavedToStorage {
		record: Rc<ListRecord>,
	},
}
impl Reducible for EditableList {
	type Action = EditableAction;

	fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
		match action {
			EditableAction::MutateData(mutator) => {
				let mut editable = (*self).clone();
				mutator(&mut editable.data);
				if editable.data == self.data {
					return self;
				}
				log::debug!("{:?}", editable.data);
				Rc::new(editable)
			}
			EditableAction::SavedToDatabase {
				record,
				save_to_storage_timeout,
			} => {
				let mut editable = (*self).clone();
				editable.record = record;
				// If a mutation occurs while we are waiting to save to storage, mark the prev timeout as cancelled.
				if let Some(cancel_prev_timeout) = editable.save_to_storage_timeout.take() {
					cancel_prev_timeout.store(true, std::sync::atomic::Ordering::Relaxed);
				}
				editable.save_to_storage_timeout = Some(save_to_storage_timeout);
				Rc::new(editable)
			}
			EditableAction::SavedToStorage { record } => {
				let mut editable = (*self).clone();
				editable.record = record;
				Rc::new(editable)
			}
		}
	}
}

#[derive(Clone, PartialEq)]
struct EditableListHandle(UseReducerHandle<EditableList>);
impl std::ops::Deref for EditableListHandle {
	type Target = UseReducerHandle<EditableList>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl EditableListHandle {
	pub fn get_route(&self, path: Option<EntryPath>) -> Route {
		Route::new_list_entry((*self.id).clone(), path)
	}

	pub fn mutate<F>(&self, mutator: F)
	where
		F: FnOnce(&mut ListData) -> Option<String> + 'static,
	{
		self.dispatch(EditableAction::MutateData(Box::new({
			let handle = self.clone();
			move |data| {
				if let Some(msg) = mutator(data) {
					let kdl_data = data.as_kdl();
					let node = kdl_data.build(ListData::id());
					let content = node.to_doc_string_unescaped();
					handle.save_to_database(msg, content);
				}
			}
		})));
	}

	fn save_to_database(self, message: String, kdl: String) {
		crate::util::spawn_local("wishlist", async move {
			let Some(mut record) = self.database.get::<ListRecord>(&self.record.id).await? else {
				return Err(anyhow::anyhow!(format!("List record {:?} went missing.", self.record.id)));
			};

			record.kdl = kdl.clone();
			record.pending_changes.push((message, kdl));

			let transaction = self.database.write()?;
			transaction.put_ref(&record).await?;
			transaction.commit().await?;

			let was_canceled = Rc::new(AtomicBool::new(false));
			let timeout = gloo_timers::callback::Timeout::new(1000 * 30, {
				let handle = self.clone();
				let was_canceled = was_canceled.clone();
				move || {
					if was_canceled.load(std::sync::atomic::Ordering::Relaxed) {
						return;
					}
					handle.save_to_storage();
				}
			});
			timeout.forget();

			self.dispatch(EditableAction::SavedToDatabase {
				record: Rc::new(record),
				save_to_storage_timeout: was_canceled,
			});

			Ok(()) as anyhow::Result<()>
		});
	}

	fn save_to_storage(self) {
		crate::util::spawn_local("wishlist", async move {
			let auth_info = yewdux::dispatch::get::<crate::auth::Info>();
			let auth_status = yewdux::dispatch::get::<crate::auth::Status>();
			let Some(storage) = crate::storage::get(&*auth_status) else {
				return Err(anyhow::anyhow!("Failed to create storage."));
			};
			let Some(mut user) = self.database.get::<crate::database::User>(auth_info.name.as_str()).await? else {
				return Err(anyhow::anyhow!(format!("Missing user record for {:?}", auth_info.name)));
			};

			let Some(mut record) = self.database.get::<ListRecord>(&self.record.id).await? else {
				return Err(anyhow::anyhow!(format!("List record {:?} went missing.", self.record.id)));
			};
			let mut pending_changes = Vec::new();
			std::mem::swap(&mut pending_changes, &mut record.pending_changes);

			// generate commits for each pending change and push them to storage.
			// there is no method to the github api for pushing multiple commits at once,
			// so we must accept the non-atomic "sequentially push each commit".
			let file_name = format!("{}.kdl", self.id.id);
			for (message, content) in pending_changes {
				let args = github::repos::contents::update::Args {
					repo_org: &auth_info.name,
					repo_name: crate::storage::USER_DATA_REPO_NAME,
					path_in_repo: std::path::Path::new(&file_name),
					commit_message: &message,
					content: &content,
					file_id: Some(&record.file_id),
					branch: None,
				};
				let response = storage.create_or_update_file(args).await?;
				record.file_id = response.file_id;
				record.local_version = response.version;
			}
			user.local_version = record.local_version.clone();

			let transaction = self.database.write()?;
			transaction.put_ref(&user).await?;
			transaction.put_ref(&record).await?;
			transaction.commit().await?;

			self.dispatch(EditableAction::SavedToStorage {
				record: Rc::new(record),
			});
			Ok(()) as anyhow::Result<()>
		});
	}

	fn force_save_to_storage(&self) {
		// If a mutation occurs while we are waiting to save to storage, mark the prev timeout as cancelled.
		if let Some(cancel_prev_timeout) = &self.save_to_storage_timeout {
			cancel_prev_timeout.store(true, std::sync::atomic::Ordering::Relaxed);
		}
		self.clone().save_to_storage();
	}

	pub fn mutate_entry<F>(&self, path: &EntryPath, mutator: F)
	where
		F: FnOnce(&mut Entry) -> Option<String> + 'static,
	{
		self.mutate({
			let path = path.clone();
			move |list| match path.resolve_mut(list) {
				Some(entry) => mutator(entry),
				None => None,
			}
		});
	}

	pub fn mutate_entry_callback<T, F>(&self, path: &EntryPath, mutator: F) -> Callback<T>
	where
		F: Fn(T, &mut Entry) -> Option<String> + 'static,
		T: 'static,
	{
		let list = self.clone();
		let path = path.clone();
		let mutator = Rc::new(mutator);
		Callback::from(move |value: T| {
			let mutator = mutator.clone();
			list.mutate_entry(&path, move |entry| mutator(value, entry));
		})
	}

	fn add_entry(&self, dst_path: EntryPath, entry: Entry) {
		self.mutate(move |data| match dst_path.bundle_idx {
			None => {
				let dst_index = dst_path.root.min(data.entries.len());
				data.entries.insert(dst_index, entry);
				Some(format!("Add item"))
			}
			Some(bundle_idx) => {
				let Some(bundle_entry) = data.entries.get_mut(dst_path.root) else {
						return None;
					};
				let Kind::Bundle(bundle) = &mut bundle_entry.kind else {
						return None;
					};
				let dst_index = bundle_idx.min(bundle.entries.len());
				bundle.entries.insert(dst_index, entry);
				Some(format!("Add item to bundle"))
			}
		});
	}

	fn remove_entry(&self, path: EntryPath) {
		self.mutate(move |data| match path.bundle_idx {
			None => {
				if data.entries.get(path.root).is_none() {
					return None;
				}
				let entry = data.entries.remove(path.root);
				Some(format!("Remove item {}", entry.name))
			}
			Some(bundle_idx) => {
				let Some(entry) = data.entries.get_mut(path.root) else {
						return None;
					};
				let Kind::Bundle(bundle) = &mut entry.kind else {
						return None;
					};
				if bundle.entries.get(bundle_idx).is_none() {
					return None;
				}
				let removed = bundle.entries.remove(bundle_idx);
				Some(format!("Remove item {} from bundle {}", removed.name, entry.name))
			}
		});
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
fn ListBody(
	ListBodyProps {
		list_id,
		record,
		data,
		entry_path,
	}: &ListBodyProps,
) -> Html {
	let database = use_context::<Database>().unwrap();
	let list = EditableListHandle(use_reducer({
		let editable = EditableList {
			id: Rc::new(list_id.clone()),
			record: Rc::new(record.clone()),
			data: data.clone(),
			database,
			save_to_storage_timeout: None,
		};
		move || editable
	}));

	let entry_at_path = match entry_path {
		None => None,
		Some(path) => match path.resolve_crumbs(&list.data) {
			None => None,
			Some(crumbs) => Some((path, crumbs)),
		},
	};

	html! {
		<ContextProvider<EditableListHandle> context={list.clone()}>
			{match entry_at_path {
				None => {
					html!(<ListContent />)
				}
				Some((path, crumbs)) => html! {
					<div class="entry page">
						<nav style="--bs-breadcrumb-divider: '>';">
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
						<EntryContent path={*path} />
					</div>
				},
			}}
		</ContextProvider<EditableListHandle>>
	}
}

#[function_component]
fn ListContent() -> Html {
	let list = use_context::<EditableListHandle>().unwrap();
	let navigator = use_navigator().unwrap();

	// TODO-QoL: maybe save to session storage?
	let tag_filter = use_state_eq(|| BTreeSet::default());

	let add_entry = Callback::from({
		let list = list.clone();
		let navigator = navigator.clone();
		move |_| {
			let dst_path = EntryPath::root(0);
			let mut entry = crate::data::Entry::default();
			entry.name = format!("CustomItem");
			list.add_entry(dst_path, entry);
			navigator.push(&list.get_route(Some(dst_path)));
		}
	});
	let delete_entry = Callback::from({
		let list = list.clone();
		move |path: EntryPath| {
			list.remove_entry(path);
		}
	});

	let invite_user = Callback::from({
		let list = list.clone();
		move |_| {}
	});
	let remove_user = Callback::from({
		let list = list.clone();
		move |_user_id: AttrValue| {}
	});

	let toggle_tag = Callback::from({
		let tag_filter = tag_filter.clone();
		move |tag: AttrValue| {
			if tag_filter.contains(&tag) {
				let mut filter = (*tag_filter).clone();
				filter.remove(&tag);
				tag_filter.set(filter);
			} else {
				let mut filter = (*tag_filter).clone();
				filter.insert(tag);
				tag_filter.set(filter);
			}
		}
	});

	html! {
		<div class="list page">
			<div class="d-flex">
				<h2>{&list.data.name}</h2>
				{(!list.record.pending_changes.is_empty()).then(|| html!(
					<div class="alert alert-warning ms-auto mb-0 p-2 d-flex align-items-center" role="alert">
						<div>{list.record.pending_changes.len()}{" pending changes to save"}</div>
						<button class="btn btn-success btn-sm ms-3" onclick={Callback::from({
							let handle = list.clone();
							move |_| handle.force_save_to_storage()
						})}>
							<i class="bi bi-floppy" />
							{"Save Now"}
						</button>
					</div>
				))}
			</div>

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
				{list.data.entries.iter().fold(BTreeSet::default(), |mut tags, entry| {
					tags.extend(entry.tags.iter().map(|s| AttrValue::from(s.clone())));
					tags
				}).into_iter().map(|tag| {
					let on_click = toggle_tag.reform({
						let tag = tag.clone();
						move |_| tag.clone()
					});
					html!(<Tag classes="is-rounded" active={tag_filter.contains(&tag)} {on_click}>
						{tag.clone()}
					</Tag>)
				}).collect::<Vec<_>>()}
			</Tags>

			{entry_cards(
				&list.id, None, &list.data.entries,
				Some(tag_filter.clone()),
				add_entry, delete_entry
			)}
		</div>
	}
}

fn entry_cards(
	list_id: &ListId,
	parent: Option<EntryPath>,
	entries: &Vec<Entry>,
	tag_filter: Option<UseStateHandle<BTreeSet<AttrValue>>>,
	add: Callback<()>,
	remove: Callback<EntryPath>,
) -> Html {
	html! {
		<div class="d-flex flex-wrap">
			<div class="card entry m-2">
				<div class="card-body d-flex align-items-center justify-content-center">
					<button class="btn btn-success" onclick={add.reform(|_| ())}>
						<i class="bi bi-plus" />
						{"Add Item"}
					</button>
				</div>
			</div>
			{entries.iter().enumerate().map(|(idx, _)| {
				let path = match parent {
					None => EntryPath::root(idx),
					Some(path) => path.bundled(idx),
				};
				let route = Route::new_list_entry(list_id.clone(), Some(path));
				let delete = remove.reform(move |_| path);
				html!(<EntryCard {path} {route} {delete} tag_filter={tag_filter.clone()} />)
			}).collect::<Vec<_>>()}
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

	fn resolve_crumbs<'list>(&self, list: &'list ListData) -> Option<Vec<(&'list String, Option<EntryPath>)>> {
		let mut crumbs = Vec::with_capacity(3);
		crumbs.push((&list.name, None));
		let Some(entry) = list.entries.get(self.root) else {
			return None;
		};
		crumbs.push((&entry.name, Some(Self::root(self.root))));
		let Some(bundle_idx) = self.bundle_idx else {
			return Some(crumbs);
		};
		let Kind::Bundle(bundle) = &entry.kind else {
			return None;
		};
		let Some(entry) = bundle.entries.get(bundle_idx) else {
			return None;
		};
		crumbs.push((&entry.name, Some(*self)));
		Some(crumbs)
	}

	fn resolve_entry<'list>(&self, list: &'list ListData) -> Option<&'list Entry> {
		let mut crumbs = Vec::with_capacity(3);
		crumbs.push((&list.name, None));
		let Some(entry) = list.entries.get(self.root) else {
			return None;
		};
		crumbs.push((&entry.name, Some(Self::root(self.root))));
		let Some(bundle_idx) = self.bundle_idx else {
			return Some(entry);
		};
		let Kind::Bundle(bundle) = &entry.kind else {
			return None;
		};
		let Some(entry) = bundle.entries.get(bundle_idx) else {
			return None;
		};
		crumbs.push((&entry.name, Some(*self)));
		Some(entry)
	}

	fn resolve_mut<'list>(&self, list: &'list mut ListData) -> Option<&'list mut Entry> {
		let Some(entry) = list.entries.get_mut(self.root) else {
			return None;
		};
		let Some(bundle_idx) = self.bundle_idx else {
			return Some(entry);
		};
		let Kind::Bundle(bundle) = &mut entry.kind else {
			return None;
		};
		bundle.entries.get_mut(bundle_idx)
	}
}

#[derive(Clone, PartialEq, Properties)]
struct EntryCardProps {
	path: EntryPath,
	route: Route,
	delete: Callback<()>,
	tag_filter: Option<UseStateHandle<BTreeSet<AttrValue>>>,
}
#[function_component]
fn EntryCard(props: &EntryCardProps) -> Html {
	let EntryCardProps {
		path,
		route,
		delete,
		tag_filter,
	} = props;
	let list = use_context::<EditableListHandle>().unwrap();
	let auth_info = use_store_value::<crate::auth::Info>();
	let Some(entry) = path.resolve_entry(&list.data) else {
		return html!("404 entry not found - todo better card");
	};
	let is_owner = list.record.id.starts_with(&auth_info.name);
	let kind_id = entry.kind.id().to_string();

	if let Some(filter) = tag_filter {
		if !filter.is_empty() {
			let filter = filter.iter().map(AttrValue::as_str).collect::<BTreeSet<_>>();
			let tags = entry.tags.iter().map(String::as_str).collect::<BTreeSet<_>>();
			if filter.intersection(&tags).count() == 0 {
				return html!();
			}
		}
	}

	let image_urls = entry.kind.image_urls();
	let image = match image_urls.len() {
		0 => html!(),
		1 => html!(<img class="card-img-top card-img" src={image_urls.into_iter().next().cloned()} />),
		_ => html!(<>{image_urls.into_iter().map(|url| {
			html!(<img src={url.clone()} class="card-img" />)
		}).collect::<Vec<_>>()}</>),
	};
	let view_at_url = match &entry.kind {
		Kind::Specific(specific) if !specific.offer_url.is_empty() => html! {
			<a class="icon-link" target="_blank" href={specific.offer_url.clone()}>
				{"View Url"}
				<i class="bi bi-box-arrow-up-right" style="height: auto;" />
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
				{is_owner.then(|| html!(<i
					class={"bi bi-trash ms-auto ps-3"}
					onclick={delete.reform(|_| ())}
				/>))}
			</div>
			<div class="d-flex justify-content-center mt-2">
				{image}
			</div>
			<div class="card-body">
				<h5 class="card-title">{&entry.name}</h5>
				<div>{&entry.description}</div>
			</div>
			<div class="card-footer d-flex">
				{view_at_url}
				<Link<Route> classes="icon-link icon-link-hover ms-auto" to={route.clone()}>
					{"Open"}
					<i class="bi bi-chevron-right" style="height: auto;" />
				</Link<Route>>
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct EntryContentProps {
	path: EntryPath,
}
#[function_component]
fn EntryContent(EntryContentProps { path }: &EntryContentProps) -> Html {
	let list = use_context::<EditableListHandle>().unwrap();
	let navigator = use_navigator().unwrap();
	let auth_info = use_store_value::<crate::auth::Info>();
	let Some(entry) = path.resolve_entry(&list.data) else {
		return html!("404 entry not found - todo better display");
	};
	let is_owner = list.record.id.starts_with(&auth_info.name);
	let form_control = match is_owner {
		true => "form-control",
		false => "form-control-plaintext",
	};

	log::debug!("{entry:?}");

	let set_name = list.mutate_entry_callback(path, |value: String, entry: &mut Entry| {
		let msg = format!("Change name of item from {:?} to {:?}", entry.name, value);
		entry.name = value;
		Some(msg)
	});
	let set_name = set_name
		.filter_reform(|evt: web_sys::Event| evt.input_value())
		.map(|_| ());

	let set_tag_active =
		list.mutate_entry_callback(path, |(value, mark_active): (AttrValue, bool), entry: &mut Entry| {
			if mark_active {
				entry.tags.insert(value.to_string());
				Some(format!("Add tag {:?} to item {:?}", value.as_str(), entry.name))
			} else {
				entry.tags.remove(value.as_str());
				Some(format!("Remove tag {:?} from item {:?}", value.as_str(), entry.name))
			}
		});
	let add_tag = Callback::from(|evt: web_sys::Event| evt.input_value());
	let add_tag = add_tag.map_some(|tag| (!tag.is_empty()).then_some(tag));
	let add_tag = add_tag
		.then_emit(set_tag_active.reform(|value: String| (value.into(), true)))
		.map(|_| ());

	let set_description = list.mutate_entry_callback(path, |value: String, entry: &mut Entry| {
		entry.description = value;
		Some(format!("Update description of item {:?}", entry.name))
	});
	let set_description = set_description
		.filter_reform(|evt: web_sys::Event| evt.input_value())
		.map(|_| ());

	let set_quantity = list.mutate_entry_callback(path, |value: usize, entry: &mut Entry| {
		entry.quantity = value.max(1);
		Some(format!("Update desired quantity of item {:?}", entry.name))
	});
	let set_quantity = set_quantity
		.filter_reform(|evt: web_sys::Event| evt.input_value_t())
		.map(|_| ());

	let set_kind = Callback::from(|evt: web_sys::Event| evt.select_value());
	let set_kind = set_kind.map_some(|id_str| KindId::from_str(&id_str).ok());
	let set_kind = set_kind
		.then_emit(list.mutate_entry_callback(path, |id: KindId, entry: &mut Entry| {
			let msg = format!(
				"Change item kind for {:?} from {:?} to {:?}",
				entry.name,
				entry.kind.id(),
				id
			);
			entry.kind = id.into();
			Some(msg)
		}))
		.map(|_| ());

	let tag_states = {
		// all tags in the list are flagged as inactive for the entry
		let unique_tags = list.data.entries.iter().fold(BTreeSet::default(), |mut tags, entry| {
			tags.extend(entry.tags.iter().map(|s| AttrValue::from(s.clone())));
			tags
		});
		let other_tags = unique_tags.into_iter();
		let other_tags = other_tags.map(|tag| (tag, false));
		// tags enabled on this entry are flagged as active
		let marked_tags = entry.tags.iter();
		let marked_tags = marked_tags.map(|tag| (AttrValue::from(tag.clone()), true));
		// and marked tags override other tags
		let tags = other_tags.chain(marked_tags);
		tags.collect::<BTreeMap<_, _>>()
	};

	// TODO: reserve btn + how many ive reserved
	// TODO: specific kind; image url, offer url, cost
	// TODO: idea kind; image url, estimated cost, example urls
	// TODO: bundle kind; entry cards/editor

	let entry_fields = html! {<>
		<div class="mb-3">
			<label for="name" class="form-label">{"Title"}</label>
			<input
				type="text"
				class={form_control} readonly={!is_owner}
				id="name" value={entry.name.clone()}
				onchange={set_name}
			/>
		</div>

		<div class="mb-3">
			<label for="name" class="form-label">{"Tags"}</label>
			{is_owner.then(move || html!(
				<div class="input-group input-group-sm mb-2" style="width: 150px;">
					<input
						type="text" class="form-control"
						id="tag-name" onchange={add_tag}
					/>
				</div>
			))}
			<Tags>
				{tag_states.into_iter().map({
					|(tag, active)| {
						let on_click = set_tag_active.reform({
							let tag = tag.clone();
							move |is_active: bool| (tag.clone(), !is_active)
						});
						let on_click = is_owner.then(move || on_click);
						html!(
							<Tag {active} {on_click}>
								{tag.clone()}
							</Tag>
						)
					}
				}).collect::<Vec<_>>()}
			</Tags>
		</div>

		<div class="mb-3">
			<label class="form-label" for="description">{"Description"}</label>
			<textarea rows="3"
				class={form_control} readonly={!is_owner}
				id="description" value={entry.description.clone()}
				onchange={set_description}
			/>
		</div>

		<div class="mb-3">
			<label for="quantity" class="form-label">{"Desired Quantity"}</label>
			<input
				type="number" class={form_control}
				id={"quantity"}
				min="1" style={"width: 100px;"}
				value={format!("{}", entry.quantity)}
				onkeydown={validate_uint_only()}
				onchange={set_quantity}
			/>
		</div>

		<div class="mb-3">
			<label class="form-label" for="kind">{"Kind"}</label>
			{match is_owner {
				true => html! {
					<select
						class={"form-select form-select-lg w-auto"}
						id="kind"
						onchange={set_kind}
					>
						{EnumSet::<KindId>::all().into_iter().map(|kind_id| {
							html!(<option
								value={kind_id.to_string()}
								selected={entry.kind.id() == kind_id}
							>
								{kind_id.to_string()}
							</option>)
						}).collect::<Vec<_>>()}
					</select>
				},
				false => html! {
					<div style="font-size: 1.25rem; font-weight: 400;">
						{entry.kind.id().to_string()}
					</div>
				},
			}}
			<div id="kindHelp" class="form-text">
				{entry.kind.id().help_info()}
			</div>
		</div>
	</>};

	let specialized_fields = match &entry.kind {
		Kind::Specific(specific) => {
			let set_image_url = list.mutate_entry_callback(path, |value: String, entry: &mut Entry| {
				let Kind::Specific(specific) = &mut entry.kind else {
					return None;
				};
				specific.image_url = (!value.is_empty()).then_some(value);
				Some(format!("Update image url for item {:?}", entry.name))
			});
			let set_image_url: Callback<Event> = set_image_url
				.filter_reform(|evt: web_sys::Event| evt.input_value())
				.map(|_| ());

			let set_offer_url = list.mutate_entry_callback(path, |value: String, entry: &mut Entry| {
				let Kind::Specific(specific) = &mut entry.kind else {
					return None;
				};
				specific.offer_url = value;
				Some(format!("Update offer url for item {:?}", entry.name))
			});
			let set_offer_url: Callback<Event> = set_offer_url
				.filter_reform(|evt: web_sys::Event| evt.input_value())
				.map(|_| ());

			let set_cost = Callback::from(|evt: web_sys::Event| evt.input_value());
			let set_cost = set_cost.map_some(|value| match value.is_empty() {
				true => Some(0),
				false => value.parse::<usize>().ok(),
			});
			let set_cost = set_cost.then_emit(list.mutate_entry_callback(path, |value: usize, entry: &mut Entry| {
				let Kind::Specific(specific) = &mut entry.kind else {
					return None;
				};
				specific.cost_per_unit = value.max(0);
				Some(format!("Update cost per unit for item {:?}", entry.name))
			}));
			let set_cost = set_cost.map(|_| ());

			html! {
				<div class="d-flex">
					<div>
						<img
							class="rounded mb-1"
							src={specific.image_url.clone()}
							style="max-width: 400px; max-height: 400px;"
						/>
						<div class="mb-3">
							<label for="image-url" class="form-label">{"Image Url"}</label>
							<input
								type="text"
								class={form_control} readonly={!is_owner}
								id="image-url" value={specific.image_url.clone()}
								onchange={set_image_url}
							/>
						</div>
					</div>
					<div class="ms-3 flex-fill">
						<div class="mb-3">
							<label for="offer-url" class="form-label">{"Offer Url"}</label>
							<input
								type="text"
								class={form_control} readonly={!is_owner}
								id="offer-url" value={specific.offer_url.clone()}
								onchange={set_offer_url}
							/>
						</div>
						<div class="mb-3">
							<label for="cost" class="form-label">{"Estimated Cost"}</label>
							<div class="input-group">
								<span class="input-group-text">{"$"}</span>
								<input
									type="number"
									class={form_control} readonly={!is_owner}
									id="cost" value={specific.cost_per_unit.to_string()}
									onkeydown={validate_uint_only()}
									onchange={set_cost}
								/>
							</div>
							<div id="costHelp" class="form-text">
								{"This is about how much it will cost per item."}
							</div>
						</div>
					</div>
				</div>
			}
		}
		Kind::Idea(idea) => {
			let set_image_url = list.mutate_entry_callback(path, |value: String, entry: &mut Entry| {
				let Kind::Idea(idea) = &mut entry.kind else {
					return None;
				};
				idea.image_url = (!value.is_empty()).then_some(value);
				Some(format!("Update image url for item {:?}", entry.name))
			});
			let set_image_url: Callback<Event> = set_image_url
				.filter_reform(|evt: web_sys::Event| evt.input_value())
				.map(|_| ());

			let set_cost = Callback::from(|evt: web_sys::Event| evt.input_value());
			let set_cost = set_cost.map_some(|value| match value.is_empty() {
				true => Some(0),
				false => value.parse::<usize>().ok(),
			});
			let set_cost = set_cost.then_emit(list.mutate_entry_callback(path, |value: usize, entry: &mut Entry| {
				let Kind::Idea(idea) = &mut entry.kind else {
					return None;
				};
				idea.estimated_cost = value.max(0);
				Some(format!("Update estimated cost per unit for item {:?}", entry.name))
			}));
			let set_cost = set_cost.map(|_| ());

			html! {
				<div class="d-flex">
					<div>
						<img
							class="rounded mb-1"
							src={idea.image_url.clone()}
							style="max-width: 400px; max-height: 400px;"
						/>
						<div class="mb-3">
							<label for="image-url" class="form-label">{"Image Url"}</label>
							<input
								type="text"
								class={form_control} readonly={!is_owner}
								id="image-url" value={idea.image_url.clone()}
								onchange={set_image_url}
							/>
						</div>
					</div>
					<div class="ms-3 flex-fill">
						<div class="mb-3">
							<label for="cost" class="form-label">{"Estimated Cost"}</label>
							<div class="input-group">
								<span class="input-group-text">{"$"}</span>
								<input
									type="number"
									class={form_control} readonly={!is_owner}
									id="cost" value={idea.estimated_cost.to_string()}
									onkeydown={validate_uint_only()}
									onchange={set_cost}
								/>
							</div>
							<div id="costHelp" class="form-text">
								{"This is about how much it might cost per item."}
							</div>
						</div>
						<div class="mb-3">
							<label for="examples" class="form-label">{"Example Urls"}</label>

						</div>
					</div>
				</div>
			}
		}
		Kind::Bundle(bundle) => {
			let add = Callback::from({
				let list = list.clone();
				let path = path.clone();
				move |_| {
					let dst_path: EntryPath = path.bundled(0);
					let mut entry = crate::data::Entry::default();
					entry.name = format!("CustomItem");
					list.add_entry(dst_path, entry);
					navigator.push(&list.get_route(Some(dst_path)));
				}
			});
			let remove = Callback::from({
				let list = list.clone();
				move |path: EntryPath| {
					list.remove_entry(path);
				}
			});
			entry_cards(&list.id, Some(*path), &bundle.entries, None, add, remove)
		}
	};

	html! {<div>
		{entry_fields}
		{specialized_fields}
	</div>}
}
