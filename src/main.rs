use yew::prelude::*;
use yew_router::prelude::*;

mod logging;
mod page;
mod theme;

#[cfg(target_family = "wasm")]
fn main() {
	logging::init(logging::Config::default().prefer_target());
	yew::Renderer::<App>::new().render();
}

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
	#[at("/")]
	Home,
	#[not_found]
	#[at("/404")]
	NotFound,
}

impl Route {
	pub fn not_found() -> Html {
		html!(<Redirect<Self> to={Self::NotFound} />)
	}

	fn switch(self) -> Html {
		match self {
			Self::Home => html!(),
			Self::NotFound => html!(<page::NotFound />),
		}
	}
}

#[function_component]
fn App() -> Html {
	html! {
		<BrowserRouter>
			<header>
				<nav class="navbar navbar-expand-lg sticky-top bg-body-tertiary">
					<div class="container-fluid">
						<Link<Route> classes={classes!("navbar-brand")} to={Route::Home}>{"Wishlist"}</Link<Route>>
						<button
							class="navbar-toggler" type="button"
							data-bs-toggle="collapse" data-bs-target="#navContent"
							aria-controls="navContent" aria-expanded="false" aria-label="Toggle navigation"
						>
							<span class="navbar-toggler-icon"></span>
						</button>
						<div class="collapse navbar-collapse" id="navContent">
							<ul class="navbar-nav">
								
							</ul>
							<ul class="navbar-nav flex-row flex-wrap ms-md-auto">
								<theme::Dropdown />
							</ul>
						</div>
					</div>
				</nav>
			</header>
			<Switch<Route> render={Route::switch} />
		</BrowserRouter>
	}
}
