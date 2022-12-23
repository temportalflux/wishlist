use crate::api::github::AccessToken;
use yew::{html, Component, Context, Html};
use yew_router::{BrowserRouter, Routable};

pub mod api;
pub mod components;
pub mod config;
pub mod index;
pub mod route;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
	#[at("api/*")]
	Api,
	#[not_found]
	#[at("")]
	Webpage,
}

impl crate::route::Route for Route {
	fn html(&self) -> Html {
		log::debug!("access token: {:?}", AccessToken::load());
		let base_url = gloo_utils::document().base_uri().ok().flatten().unwrap();
		log::debug!("base_url: {base_url}");
		match self {
			Self::Api => <api::Route as route::Route>::switch(),
			Self::Webpage => html! { <index::Page /> },
		}
	}
}

pub struct Root;
impl Component for Root {
	type Message = ();
	type Properties = ();

	fn create(_ctx: &Context<Self>) -> Self {
		Self
	}

	#[allow(unused_parens)]
	fn view(&self, _ctx: &Context<Self>) -> Html {
		html! {
			<BrowserRouter>
				{ <Route as route::Route>::switch() }
			</BrowserRouter>
		}
	}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::start_app::<Root>();
}
