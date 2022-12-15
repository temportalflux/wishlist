use yew::{html, Component, Context, Html};

struct Index;
impl Component for Index {
	type Message = ();
	type Properties = ();

	fn create(_ctx: &Context<Self>) -> Self {
		Self
	}

	fn view(&self, _ctx: &Context<Self>) -> Html {
		html! {
			<div>
				{ "Hello, World!" }
				<br />
				{ "This is some new text" }
			</div>
		}
	}
}

fn main() {
	yew::Renderer::<Index>::new().render();
}
