use crate::{
	api::github::gist::{self},
	components::AuthSwitch,
	session::{Profile, SessionValue},
};
use ybc::{Button, CardContent, CardFooter, CardHeader, Content, Section, Title, Field, Control};
use yew::{prelude::*};
use yew_hooks::{
	use_drag_with_options, use_drop_with_options, use_session_storage, UseDragOptions,
	UseDropOptions, use_async,
};

#[function_component]
pub fn Home() -> Html {
	let profile_handle = use_session_storage::<Profile>(Profile::id().to_string());
	let fetch_profile_handle = use_async(async move {
		match gist::FetchProfile::get().await {
			Ok(profile) => {
				log::debug!("received updated profile");
				profile_handle.set(profile)
			},
			Err(err) => log::debug!("{err:?}"),
		}
		Ok(()) as Result<(), ()>
	});
	let create_private = Callback::from(move |_| {
		fetch_profile_handle.run();
	});
	let create_wishlist = Callback::from(|_| {
		let user = crate::session::User::load().unwrap();
		wasm_bindgen_futures::spawn_local(async {
			let description = "Test List".to_owned();
			let list = gist::List::new(description.clone());
			let mut gist = list.into_gist();
			let result = gist.save().await;
			log::debug!("{result:?}");
			if result.is_ok() {
				let mut profile = Profile::load().unwrap();
				profile.lists.push(gist::GistInfo {
					id: gist.id.unwrap(),
					title: description,
					owner_login: user.login,
				});
				profile.apply_to_session();
			}
		});
	});
	html! {<>
		<AuthSwitch
			identified={html! {
				<Section>
					<Field grouped=true>
						<Control>
							<Title>{"My Wishlists"}</Title>
						</Control>
						<Control>
							<button class={"button is-primary is-small align-v"} onclick={create_wishlist}>{"+ New"}</button>
						</Control>
					</Field>
					<Button onclick={create_private}>{"Fetch gist data"}</Button>
					<ProfileWishlistCardGrid />
					<div class={"container"} style={"display: grid; grid-template-columns: repeat(auto-fill, minmax(250px,1fr)); grid-gap: 0.5em;"}>
						<WishlistCard gist_id="0" title={"List A"} />
						<WishlistCard gist_id="1" title={"List B"} />
						<WishlistCard gist_id="2" title={"List C"} />
						<WishlistCard gist_id="3" title={"List D"} />
						<WishlistCard gist_id="4" title={"List E"} />
						<WishlistCard gist_id="5" title={"List F"} />
					</div>
				</Section>
			}}
		/>
	</>}
}

#[function_component]
pub fn ProfileWishlistCardGrid() -> Html {
	let profile = use_session_storage::<Profile>(Profile::id().to_string());
	log::debug!("render wishlist grid: {:?}", *profile);
	html! {}
}

#[derive(Properties, PartialEq)]
pub struct WishlistCardProps {
	pub gist_id: String,
	pub title: String,
}

#[function_component]
pub fn WishlistCard(props: &WishlistCardProps) -> Html {
	let node = use_node_ref();

	let drag = use_drag_with_options(
		node.clone(),
		UseDragOptions {
			ondragstart: Some({
				let gist_id = props.gist_id.clone();
				Box::new(move |e| {
					if let Some(data_transfer) = e.data_transfer() {
						let _ = data_transfer.set_data("gist_id", &gist_id);
					}
				})
			}),
			..Default::default()
		},
	);
	let drop = use_drop_with_options(
		node.clone(),
		UseDropOptions {
			ondrop: Some({
				let gist_id = props.gist_id.clone();
				Box::new(move |e| {
					if let Some(data_transfer) = e.data_transfer() {
						if let Ok(other_gist_id) = data_transfer.get_data("gist_id") {
							log::debug!("Dropped {other_gist_id:?} on {gist_id:?}");
						}
					}
				})
			}),
			..Default::default()
		},
	);
	let style = match (*drag.dragging, *drop.over) {
		(true, _) => "background-color: #dedede;",
		(_, true) => "background-color: #b9ddff;",
		_ => "",
	};
	html! {
		<div class={"card"} ref={node} style={style}>
			<CardHeader>
				<p class="card-header-title">{&props.title}</p>
			</CardHeader>
			<CardContent>
				<Content>
					{"Card content"}
					<br />
					{*drag.dragging}
				</Content>
			</CardContent>
			<CardFooter>
				<a href="#" class="card-footer-item">{"Open"}</a>
				<a href="#" class="card-footer-item">{"Share"}</a>
			</CardFooter>
		</div>
	}
}
