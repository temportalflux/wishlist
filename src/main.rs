use yew::{function_component, html, Callback, Classes, Html};
use yew_router::{prelude::use_navigator, BrowserRouter, Routable};

pub mod api;
pub mod components;
pub mod config;
pub mod index;
pub mod route;

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
		log::debug!("access token: {:?}", api::github::AuthStatus::load());
		let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();
		log::debug!("base_url: {base_url}");
		log::debug!(
			"path: {:?}",
			gloo_utils::window().location().pathname().ok()
		);
		match self {
			Self::Api => <api::Route as route::Route>::switch(),
			Self::Webpage => html! { <index::Page /> },
		}
	}
}

#[function_component(Root)]
fn root_comp() -> Html {
	if let Some(nav) = use_navigator() {
		log::debug!("nav basename: {:?}", nav.basename());
	} else {
		log::debug!("no nav");
	}
	html! {
		<BrowserRouter>
			{ <Route as route::Route>::switch() }
			<AuthModal />
		</BrowserRouter>
	}
}

#[function_component]
fn AuthModal() -> Html {
	use api::github::AuthStatus;
	let mut classes = Classes::from("modal");
	let mut subtitle = html! {};
	let mut progress = html! {};
	let mut info = html! {};
	let mut close_button = html! {};
	let refresh_modal = yew::use_force_update();
	let close_modal = Callback::from(move |_| {
		AuthStatus::delete();
		refresh_modal.force_update();
	});
	if let Some(status) = AuthStatus::load() {
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
