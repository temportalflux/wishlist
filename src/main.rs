use crate::api::github::AccessToken;
use yew::{html, Html, function_component};
use yew_router::{BrowserRouter, Routable, prelude::use_navigator};

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
		log::debug!("access token: {:?}", AccessToken::load());
		let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();
		log::debug!("base_url: {base_url}");
		log::debug!("path: {:?}", gloo_utils::window().location().pathname().ok());
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
	}
	else {
		log::debug!("no nav");
	}
	html! {
		<BrowserRouter>
			{ <Route as route::Route>::switch() }
		</BrowserRouter>
	}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::Renderer::<Root>::new().render();
}
