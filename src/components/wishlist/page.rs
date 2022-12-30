use std::rc::Rc;
use crate::api::github::{
	gist::{self, GistId},
};
use ybc::{Container, Content, Icon, Title, Section, Subtitle, Tags, Tag, Button, Size, Notification};
use yew::prelude::*;
use yew_hooks::{use_mount, UseAsyncState, use_clipboard};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct PageProps {
	pub gist_id: GistId,
}

pub struct AsyncHandle<T, E> {
	state: UseStateHandle<UseAsyncState<T, E>>,
	run: Rc<dyn Fn()>,
}
impl<T, E> AsyncHandle<T, E> {
	pub fn run(&self) {
		(*self.run)();
	}
}
impl<T, E> std::ops::Deref for AsyncHandle<T, E> {
	type Target = UseAsyncState<T, E>;

	fn deref(&self) -> &Self::Target {
		&self.state
	}
}

#[hook]
fn use_async_noclone<F, T, E>(run_first_mount: bool, make_future: F) -> AsyncHandle<T, E>
where
	F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>> + 'static,
	T: 'static,
	E: 'static,
{
	let state = use_state(|| UseAsyncState {
		loading: false,
		data: None,
		error: None,
	});
	let make_future = Rc::new(make_future);
	let run = {
		let state = state.clone();
		Rc::new(move || {
			state.set(UseAsyncState {
				loading: true,
				data: None,
				error: None,
			});
			let async_state = state.clone();
			let make_future = make_future.clone();
			wasm_bindgen_futures::spawn_local(async move {
				let final_state = match make_future().await {
					Ok(data) => UseAsyncState {
						loading: false,
						data: Some(data),
						error: None,
					},
					Err(err) => UseAsyncState {
						loading: false,
						data: None,
						error: Some(err),
					},
				};
				async_state.set(final_state);
			})
		})
	};
	let run_on_mount = run.clone();
	use_mount(move || {
		if run_first_mount {
			run_on_mount();
		}
	});
	AsyncHandle { state, run }
}

#[function_component]
pub fn Page(props: &PageProps) -> Html {
	let clipboard = use_clipboard();
	let state = use_state(|| gist::Gist::<gist::List>::default());
	let saved = use_state(|| gist::Gist::<gist::List>::default());

	let fetch = {
		let handle = (state.clone(), saved.clone());
		let fetch_id = props.gist_id.clone();
		let fetch = use_async_noclone(false, move || {
			let (state, saved) = handle.clone();
			let gist_id = fetch_id.clone();
			Box::pin(async move {
				let gist = gist::Gist::<gist::List>::fetch(gist_id).await?;
				state.set(gist.clone());
				saved.set(gist);
				Ok(()) as anyhow::Result<()>
			})
		});
		// If there is no fetch in progress and we need to fetch, then run the async op.
		if !fetch.loading {
			let should_fetch = match (&state.id, &props.gist_id) {
				(None, _) => true,
				(Some(loaded), desired) if loaded != desired => true,
				_ => false,
			};
			if should_fetch {
				fetch.run();
				state.set(gist::Gist::default());
				saved.set(gist::Gist::default());
			}
		}
		fetch
	};
	if state.id.is_none() || fetch.loading {
		return html! {
			<Container>
				<ybc::Box>
					<Icon size={ybc::Size::Large}>
						<i class="fas fa-circle-notch fa-spin" />
					</Icon>
					<span>{"Fetching wishlist"}</span>
				</ybc::Box>
			</Container>
		};
	}

	let copy_link_to_clipboard = {
		let clipboard = clipboard.clone();
		let absolute_route = crate::to_abs_route(props.gist_id.as_route());
		Callback::from(move |_| {
			clipboard.write_text(absolute_route.clone());
		})
	};

	let mut save_changes_notif_classes = classes!{"m-5"};
	if *state == *saved {
		save_changes_notif_classes.push("is-hidden");
	}

	let delete_wishlist = Callback::from(|_| {
		log::debug!("TODO: Prompt delete modal");
	});
	let save_changes = Callback::from(|_| {
		log::debug!("TODO: Save to gists");
	});

	html! {<>
		<Section>
			<Container classes={"is-flex is-flex-grow-1 is-flex-shrink-0"}>
				<div class="is-justify-content-left mr-auto">
					<Title>
						{saved.file.name.clone()}
						{saved.visibility.as_tag(Some(classes!{"ml-2"}))}
						<Tag tag="a" classes={"ml-2"} onclick={copy_link_to_clipboard}>{"Copy Link"}</Tag>
					</Title>
					<Subtitle>{"Owned By: "}{saved.owner_login.clone()}</Subtitle>
				</div>
				<div class="is-justify-content-right ml-auto">
					<Button classes={"is-danger is-light"} onclick={delete_wishlist}>
						<Icon size={Size::Small}><i class="fas fa-trash" /></Icon>
						<span>{"Delete"}</span>
					</Button>
				</div>
			</Container>
			<div class={save_changes_notif_classes}>
				<Notification classes={"is-success is-light"}>
					<Content classes="has-text-centered">
						{"You have unsaved changes to your wishlist!"}
						<br />
						<Button classes={"is-primary"} onclick={save_changes}>{"Save Changes"}</Button>
					</Content>
				</Notification>
			</div>
			<Tags classes={"is-justify-content-center"}>
				<Tag classes={"is-rounded is-info"}>{"All"}</Tag>
				<Tag classes={"is-rounded"}>{"Category 1"}</Tag>
				<Tag classes={"is-rounded"}>{"Category 2"}</Tag>
				<Tag classes={"is-rounded"}>{"Category 3"}</Tag>
			</Tags>
		</Section>
	</>}
}
