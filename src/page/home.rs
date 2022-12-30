use crate::{
	api::github::gist::{self, GistInfo, Visibility},
	components::{
		wishlist::{InfoModalPrompt, InfoModalProps},
		AuthSwitch,
	},
	page::Route,
	session::{Profile, User},
};
use ybc::{
	Button, ButtonGroupSize, Buttons, CardContent, CardFooter, CardHeader, Content, Control, Field,
	Icon, Section, Size, Title,
};
use yew::prelude::*;
use yew_hooks::{
	use_async, use_clipboard, use_drag_with_options, use_drop_with_options, UseDragOptions,
	UseDropOptions,
};
use yew_router::prelude::Link;
use yewdux::prelude::{use_store, Dispatch};

#[function_component]
pub fn Home() -> Html {
	let fetch_profile_handle = use_async(async move {
		match gist::FetchProfile::get().await {
			Ok(profile) => Dispatch::<Profile>::new().set(profile),
			Err(err) => log::debug!("{err:?}"),
		}
		Ok(()) as Result<(), ()>
	});
	let refresh_profile = {
		let async_run = fetch_profile_handle.clone();
		Callback::from(move |_| async_run.run())
	};

	let create_wishlist = Dispatch::<InfoModalPrompt>::new().reduce_mut_callback(|active| {
		*active = InfoModalProps {
			id: None,
			title: "".into(),
			owner_login: Dispatch::<User>::new().get().login.clone(),
			visibility: Visibility::Public,
		}
		.into();
	});
	let mut refresh_icon = classes! {"fas", "fa-arrows-rotate"};
	if fetch_profile_handle.loading {
		refresh_icon.push("fa-spin");
	}
	html! {<>
		<AuthSwitch
			identified={html! {
				<Section>
					<Field grouped=true>
						<Control>
							<Title>{"My Wishlists"}</Title>
						</Control>
						<Control>
							<Buttons size={ButtonGroupSize::Small}>
								<Button classes={"is-primary"} onclick={create_wishlist}>
									<Icon size={Size::Small}><i class="fas fa-plus" /></Icon>
									<span>{"New"}</span>
								</Button>
								<Button onclick={refresh_profile}>
									<Icon size={Size::Small}><i class={refresh_icon} /></Icon>
								</Button>
							</Buttons>
						</Control>
					</Field>
					<ProfileWishlistCardGrid />
				</Section>
			}}
		/>
	</>}
}

#[function_component]
pub fn ProfileWishlistCardGrid() -> Html {
	let (profile, _dispatch) = use_store::<Profile>();
	let content = match profile.lists.is_empty() {
		true => html! {{"You have no wishlists"}},
		false => html! {<> {profile.lists.iter().map(|info| {
			html! {<WishlistCard info={info.clone()} />}
		}).collect::<Vec<_>>()} </>},
	};
	html! {
		<div class={"content"} style={"display: grid; grid-template-columns: repeat(auto-fill, minmax(250px,1fr)); grid-gap: 0.5em;"}>
			{content}
		</div>
	}
}

#[derive(Properties, PartialEq)]
pub struct WishlistCardProps {
	pub info: GistInfo,
}

#[function_component]
pub fn WishlistCard(props: &WishlistCardProps) -> Html {
	let clipboard = use_clipboard();
	let absolute_route = crate::to_abs_route(props.info.id.as_route());
	let copy_route_to_clipboard = Callback::from(move |_| {
		clipboard.write_text(absolute_route.clone());
	});
	html! {
		<div class={"card"}>
			<CardHeader>
				<p class="card-header-title">{&props.info.title}</p>
			</CardHeader>
			<CardContent>
				<Content>
					{"Owner: "}{props.info.owner_login.clone()}
					<br/>
					{"Visibility: "}{props.info.visibility}
				</Content>
			</CardContent>
			<CardFooter>
				<Link<Route> classes="card-footer-item" to={props.info.id.as_route()}>{"Open"}</Link<Route>>
				<a class="card-footer-item" onclick={copy_route_to_clipboard}>{"Copy Link"}</a>
			</CardFooter>
		</div>
	}
}

// TODO: Convert this into an "ItemCard", which can be ordered within a wishlist
#[function_component]
pub fn DragableWishlistCard(props: &WishlistCardProps) -> Html {
	let node = use_node_ref();

	let drag = use_drag_with_options(
		node.clone(),
		UseDragOptions {
			ondragstart: Some({
				let gist_id = props.info.id.clone();
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
				let gist_id = props.info.id.clone();
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
				<p class="card-header-title">{&props.info.title}</p>
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
