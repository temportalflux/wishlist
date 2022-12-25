use yew::prelude::*;

use crate::api::github::{CurrentUser, Query};

#[function_component]
pub fn NameLabel() -> Html {
	html! {
		{"Name"}
	}
}
