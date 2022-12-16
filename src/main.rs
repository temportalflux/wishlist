use yew::{html, Component, Context, Html};
use yew_oauth2::oauth2::OAuth2;
use yew_router::{BrowserRouter, Routable};

pub mod api;
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
		let config = yew_oauth2::oauth2::Config {
			client_id: "479587a8a2c453a44643".to_owned(),
			auth_url: "https://github.com/login/oauth/authorize".to_owned(),
			token_url: "https://github.com/login/oauth/access_token".to_owned(),
		};
		html! {
			<BrowserRouter>
				<OAuth2 {config} scopes={vec!["gist".to_owned()]}>
					{ <Route as route::Route>::switch() }
				</OAuth2>
			</BrowserRouter>
		}
	}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::start_app::<Root>();
}
