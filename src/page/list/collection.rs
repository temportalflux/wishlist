use crate::{
	data::List,
	database::{query::use_query_all, List as ListRecord, ListId},
	GeneralProp, Route,
};
use anyhow::anyhow;
use itertools::Itertools;
use kdlize::{ext::NodeExt, AsKdl, NodeId};
use std::str::FromStr;
use yew::prelude::*;
use yew_router::prelude::use_navigator;
use yewdux::prelude::use_store_value;

fn sort_lists(a: &(ListId, &ListRecord, &List), b: &(ListId, &ListRecord, &List)) -> std::cmp::Ordering {
	let cmp_name = a.2.name.cmp(&b.2.name);
	let cmp_id = a.0.cmp(&b.0);
	cmp_name.then(cmp_id)
}

#[function_component]
pub fn Collection() -> HtmlResult {
	let query_lists = use_query_all::<crate::data::List>()?;
	let refresh_lists = Callback::from({
		let query_lists = query_lists.get_trigger().clone();
		move |_| {
			(*query_lists)(());
		}
	});
	Ok(html! {
		<div class="d-flex flex-column list-collection">
			<div class="d-flex justify-content-center">
				<ButtonCreateList value={refresh_lists.clone()} />
			</div>
			<div class="d-flex">
				{match query_lists.status() {
					Err(err) => {
						log::error!(target: "wishlist", "{err:?}");
						html!("No wishlists TODO improve this display")
					}
					Ok(lists) => {
						let iter = lists.iter();
						let iter = iter.filter_map(|(id, (record, list))| match ListId::from_str(&id) {
							Ok(id) => Some((id, record, list)),
							_ => None,
						});
						let iter = iter.sorted_by(sort_lists);
						let htmls = iter.map(|(id, record, list)| {
							html!(<ListCard
								{id}
								file_id={record.file_id.clone()}
								name={list.name.clone()}
								on_deleted={refresh_lists.clone()}
							/>)
						}).collect::<Vec<_>>();
						html!(<>{htmls}</>)
					}
				}}
			</div>
		</div>
	})
}

#[function_component]
fn ButtonCreateList(GeneralProp { value }: &GeneralProp<Callback<()>>) -> Html {
	let auth_status = use_store_value::<crate::auth::Status>();
	let auth_info = use_store_value::<crate::auth::Info>();
	let database = use_context::<crate::database::Database>().unwrap();
	let onclick = Callback::from({
		let on_success = value.clone();
		move |_| {
			let auth_status = auth_status.clone();
			let auth_info = auth_info.clone();
			let database = database.clone();
			let on_success = on_success.clone();
			crate::util::spawn_local("wishlist", async move {
				let list_id = database.new_list_id(&auth_info.name).await?;
				log::debug!("create wishlist {list_id:?}");

				let Some(storage) = crate::storage::get(&*auth_status) else {
					return Err(anyhow!("Failed to create storage."));
				};
				let Some(mut user) = database.get::<crate::database::User>(auth_info.name.as_str()).await? else {
					return Err(anyhow!(format!("Missing user record for {:?}", auth_info.name)));
				};

				let mut data = crate::data::List::default();
				data.name = format!("{}'s Wishlist", auth_info.name);
				let content = data.as_kdl().build(data.get_id()).to_doc_string_unescaped();
				let file_name = format!("{}.kdl", list_id.id);
				let message = format!("Create list {}", list_id.id);
				let args = github::repos::contents::update::Args {
					repo_org: &auth_info.name,
					repo_name: crate::storage::USER_DATA_REPO_NAME,
					path_in_repo: std::path::Path::new(&file_name),
					commit_message: &message,
					content: &content,
					file_id: None,
					branch: None,
				};
				let response = storage.create_or_update_file(args).await?;
				user.local_version = response.version.clone();
				let record = ListRecord {
					id: list_id.to_string(),
					file_id: response.file_id,
					kdl: content,
					local_version: response.version,
					pending_changes: Vec::new(),
				};

				database.write()?.put(&user).await?.put(&record).await?.commit().await?;

				on_success.emit(());
				Ok(()) as Result<(), anyhow::Error>
			});
		}
	});
	html!(<button class="btn btn-success btn-sm" {onclick}>
		<i class="bi bi-plus" />
		{"Create Wishlist"}
	</button>)
}

#[derive(Clone, PartialEq, Properties)]
struct ListCardProps {
	id: ListId,
	file_id: AttrValue,
	name: AttrValue,
	on_deleted: Callback<()>,
}
#[function_component]
fn ListCard(props: &ListCardProps) -> Html {
	let auth_info = use_store_value::<crate::auth::Info>();
	let auth_status = use_store_value::<crate::auth::Status>();
	let database = use_context::<crate::database::Database>().unwrap();
	let navigator = use_navigator().unwrap();
	let delete_list = Callback::from({
		let auth_info = auth_info.clone();
		let database = database.clone();
		let list_id = props.id.clone();
		let file_id = props.file_id.clone();
		let on_deleted = props.on_deleted.clone();
		move |_| {
			let auth_info = auth_info.clone();
			let auth_status = auth_status.clone();
			let database = database.clone();
			let list_id = list_id.clone();
			let file_id = file_id.clone();
			let on_deleted = on_deleted.clone();
			crate::util::spawn_local("wishlist", async move {
				let Some(mut user) = database.get::<crate::database::User>(auth_info.name.as_str()).await? else {
					return Err(anyhow!(format!("Missing user record for {:?}", auth_info.name)));
				};
				let Some(storage) = crate::storage::get(&*auth_status) else {
					return Err(anyhow!("Failed to create storage."));
				};
				let file_name = format!("{}.kdl", list_id.id);
				let message = format!("Delete list {}", list_id.id);
				let args = github::repos::contents::delete::Args {
					repo_org: &list_id.owner,
					repo_name: crate::storage::USER_DATA_REPO_NAME,
					path_in_repo: std::path::Path::new(&file_name),
					commit_message: &message,
					file_id: file_id.as_str(),
					branch: None,
				};
				user.local_version = storage.delete_file(args).await?;
				database
					.write()?
					.put(&user)
					.await?
					.delete::<ListRecord>(list_id.to_string())
					.await?
					.commit()
					.await?;
				on_deleted.emit(());
				Ok(()) as Result<(), anyhow::Error>
			});
		}
	});
	html! {
		<div class="card list">
			<div class="card-header d-flex align-items-center">
				<span>{&props.name}</span>
				<i
					class={"bi bi-trash ms-auto ps-3"}
					onclick={delete_list}
				/>
			</div>
			<div class="card-body">
			</div>
			<div class="card-footer">
				<button class="btn btn-primary" onclick={Callback::from({
					let route = Route::List { owner: props.id.owner.clone(), id: props.id.id.clone() };
					move |_| navigator.push(&route)
				})}>
					{"Open"}
				</button>
			</div>
		</div>
	}
}
