use yew::{html, Component, Context, Html};
use yew_router::Routable;

pub struct Switch<T>(std::marker::PhantomData<T>);
impl<T> Component for Switch<T>
where
	T: Routable + Route + 'static,
{
	type Message = ();
	type Properties = ();

	fn create(_ctx: &Context<Self>) -> Self {
		Self(Default::default())
	}

	fn view(&self, _ctx: &Context<Self>) -> Html {
		html! {
			<yew_router::Switch<T> render={T::html} />
		}
	}
}

pub trait Route {
	fn html(self) -> Html;

	fn switch() -> Html
	where
		Self: Routable + 'static,
	{
		html! { <Switch<Self> /> }
	}
}
