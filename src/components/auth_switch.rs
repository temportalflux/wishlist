use crate::session::AuthStatus;
use yew::{prelude::*, Properties};
use yewdux::prelude::use_store;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct AuthSwitchProps {
	#[prop_or_default]
	pub identified: Option<Html>,
	#[prop_or_default]
	pub anonymous: Option<Html>,
}

#[function_component]
pub fn AuthSwitch(props: &AuthSwitchProps) -> Html {
	let (auth_status, _dispatch) = use_store::<AuthStatus>();
	let empty = || html! {};
	match *auth_status {
		AuthStatus::Successful(_) => props.identified.clone().unwrap_or_else(empty),
		_ => props.anonymous.clone().unwrap_or_else(empty),
	}
}
