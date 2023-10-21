use yew::prelude::*;
use yewdux::prelude::*;

pub use netlify_oauth::*;

static SITE_ID: &str = "64a0719f-a7e3-4d4a-b538-577bf4cdf1c4";

pub fn request() -> Request {
	Request {
		site_id: SITE_ID,
		provider_id: "github",
		window_title: "Wishlist Authorization".into(),
	}
}

#[hook]
pub fn use_on_auth_success<F>(callback: F)
where
	F: Fn(&std::rc::Rc<Status>) + 'static,
{
	let callback = yew_hooks::use_latest(callback);
	let auth_status = use_store_value::<Status>();
	let was_success = use_state_eq({
		let auth_status = auth_status.clone();
		move || matches!(*auth_status, Status::Successful { .. })
	});
	use_effect_with((auth_status, was_success), move |(status, was_authenticated)| {
		let is_authenticated = matches!(**status, Status::Successful { .. });
		if is_authenticated && !**was_authenticated {
			(*callback.current())(status);
		}
		was_authenticated.set(is_authenticated);
	});
}

#[function_component]
pub fn LoginButton() -> Html {
	let auth = use_context::<Auth>().unwrap();
	let auth_status = use_store_value::<Status>();
	let autosync_status = use_context::<crate::storage::autosync::Status>().unwrap();
	let disabled = autosync_status.is_active();
	if matches!(*auth_status, Status::Successful { .. }) {
		let onclick = auth.logout_callback().reform(|_: MouseEvent| ());
		html! {
			<button class="btn btn-outline-danger" {onclick} {disabled}>
				{"Sign Out"}
			</button>
		}
	} else {
		let onclick = auth.login_callback().reform(|_: MouseEvent| request());
		html! {
			<button class="btn btn-success" {onclick} {disabled}>
				{"Sign In"}
			</button>
		}
	}
}
