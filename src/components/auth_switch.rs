use yew::{
	prelude::*,
	Component, Properties,
};
use crate::api::github::AccessToken;

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
		match AccessToken::load().is_some() {
			true => ctx.props().identified.clone().unwrap_or_else(empty),
			false => ctx.props().anonymous.clone().unwrap_or_else(empty),
		}
	}
}
