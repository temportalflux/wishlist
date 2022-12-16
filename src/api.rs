use yew::Html;
use yew_router::Routable;

pub mod auth;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
	#[at("api/auth/*")]
	Authorization,
}

impl crate::route::Route for Route {
	fn html(&self) -> Html {
		match self {
			Self::Authorization => <auth::Route as crate::route::Route>::switch(),
		}
	}
}
