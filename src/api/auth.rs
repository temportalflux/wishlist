use yew::{html, Html};
use yew_router::Routable;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
	#[at("api/auth/login")]
	Login,
	#[at("api/auth/logout")]
	Logout,
	#[at("api/auth/login_token?code={code}")]
	TokenExchange { code: String },
	#[at("api/auth/access_token?access_token={token}")]
	AccessToken { token: String },
}

impl crate::route::Route for Route {
	fn html(&self) -> Html {
		match self {
			Self::Login => html! {
				{"login"}
			},
			Self::Logout => html! {
				{"logout"}
			},
			Self::TokenExchange { code: _ } => html! {},
			Self::AccessToken { token: _ } => html! {},
		}
	}
}

impl yew::html::IntoPropValue<Option<String>> for Route {
	fn into_prop_value(self) -> Option<String> {
		Some(self.to_path())
	}
}
