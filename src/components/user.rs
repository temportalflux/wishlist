use crate::session::User;
use ybc::{Block, Image};
use yew::prelude::*;
use yewdux::prelude::use_store;

#[function_component]
pub fn Identification() -> Html {
	let (user, _dispatch) = use_store::<User>();
	html! {<>
		<Image size={Some(ybc::ImageSize::Is32x32)}>
			<img class="is-rounded" src={user.image_url.clone()} />
		</Image>
		<Block>
			<p class="is-size-6">{user.name.clone()}</p>
			<p class="is-size-7">{user.login.clone()}</p>
		</Block>
	</>}
}
