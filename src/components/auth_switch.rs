use crate::api::github::{AuthStatus, SessionValue};
use yew::{prelude::*, Component, Properties};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct AuthSwitchProps {
	#[prop_or_default]
	pub identified: Option<Html>,
	#[prop_or_default]
	pub anonymous: Option<Html>,
}

pub struct AuthSwitch;
impl Component for AuthSwitch {
	type Message = ();
	type Properties = AuthSwitchProps;

	fn create(_ctx: &yew::Context<Self>) -> Self {
		Self
	}

	fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
		let empty = || html! {};
		match AuthStatus::load() {
			Some(AuthStatus::Successful(_)) => ctx.props().identified.clone().unwrap_or_else(empty),
			_ => ctx.props().anonymous.clone().unwrap_or_else(empty),
		}
	}
}
