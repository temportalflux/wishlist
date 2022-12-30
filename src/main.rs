use session::AuthStatus;
use yew::{function_component, html, Classes, Html};
use yew_router::{BrowserRouter, Routable};
use yewdux::prelude::use_store;
use crate::components::wishlist;

pub mod api;
pub mod components;
pub mod config;
pub mod page;
pub mod response;
pub mod route;
pub mod session;

pub fn base_url() -> url::Url {
	url::Url::parse(&gloo_utils::document().base_uri().ok().flatten().unwrap()).unwrap()
}

pub fn to_abs_route(route: impl Routable) -> String {
	let url = base_url().join(&route.to_path()).unwrap();
	url.as_str().to_owned()
}

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
	#[at("/api/*")]
	Api,
	#[not_found]
	#[at("/")]
	Webpage,
}

impl crate::route::Route for Route {
	fn html(self) -> Html {
		match self {
			Self::Api => <api::Route as route::Route>::switch(),
			Self::Webpage => html! { <page::Page /> },
		}
	}
}

#[function_component(Root)]
fn root_comp() -> Html {
	html! {
		<BrowserRouter>
			{ <Route as route::Route>::switch() }
			<AuthModal />
			<wishlist::InfoModal />
		</BrowserRouter>
	}
}

#[function_component]
fn AuthModal() -> Html {
	let (status, dispatch) = use_store::<AuthStatus>();
	let mut classes = Classes::from("modal");
	let mut subtitle = html! {};
	let mut progress = html! {};
	let mut info = html! {};
	let mut close_button = html! {};
	let close_modal = dispatch.reduce_mut_callback(|status| {
		*status = AuthStatus::None;
	});
	if *status != AuthStatus::None {
		if status.should_show_modal() {
			classes.push("is-active");
		}
		subtitle = html! {<ybc::Subtitle>{status.byline()}</ybc::Subtitle>};
		progress = if status.should_show_progress() {
			html! {<progress class={"progress is-large is-info"}></progress>}
		} else {
			html! {}
		};
		info = match status.info() {
			None => html! {},
			Some(content) => {
				html! {
					<ybc::Message classes={"is-danger"}>
						<ybc::MessageBody>
							<p class={"is-size-7"}>{content}</p>
						</ybc::MessageBody>
					</ybc::Message>
				}
			}
		};
		close_button = html! {
			<button class="modal-close is-large" aria-label="close" onclick={close_modal} />
		};
	}
	html! {
		<div class={classes}>
			<div class="modal-background"></div>
			<div class="modal-content">
				<ybc::Box>
					<ybc::Title>{"Authentication"}</ybc::Title>
					{subtitle}
					{info}
					{progress}
				</ybc::Box>
			</div>
			{close_button}
		</div>
	}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::Renderer::<Root>::new().render();
}
