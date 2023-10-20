use yew::prelude::*;

#[cfg(target_family = "wasm")]
fn main() {
	logging::wasm::init(logging::wasm::Config::default().prefer_target());
	yew::Renderer::<App>::new().render();
}

#[function_component]
fn App() -> Html {
	html! {<>

	</>}
}
