use yew::prelude::*;

mod logging;

#[cfg(target_family = "wasm")]
fn main() {
	logging::init(logging::Config::default().prefer_target());
	yew::Renderer::<App>::new().render();
}

#[function_component]
fn App() -> Html {
	html! {<>

	</>}
}
