use crate::api::github::{SessionValue, User};
use ybc::{Block, Image};
use yew::prelude::*;

#[function_component]
pub fn Identification() -> Html {
	let Some(user) = User::load() else {
		return html! {};
	};
	html! {<>
		<Image size={Some(ybc::ImageSize::Is32x32)}>
			<img class="is-rounded" src={user.image_url} />
		</Image>
		<Block>
			<p class="is-size-6">{user.name}</p>
			<p class="is-size-7">{user.login}</p>
		</Block>
	</>}
}
