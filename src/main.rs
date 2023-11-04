use yew::prelude::*;
use yew_router::prelude::*;

mod auth;
mod data;
mod database;
mod logging;
mod page;
mod storage;
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
			Self::Home => html!("this is the home page"),
			Self::NotFound => html!(<page::NotFound />),
		}
	}
}

#[function_component]
fn App() -> Html {
	html! {
		<ProviderChain>
			<BrowserRouter>
				<Body />
			</BrowserRouter>
		</ProviderChain>
	}
}

#[function_component]
fn ProviderChain(props: &html::ChildrenProps) -> Html {
	html! {
		<auth::Provider>
			<database::Provider>
				<storage::autosync::Provider>
					{props.children.clone()}
				</storage::autosync::Provider>
			</database::Provider>
		</auth::Provider>
	}
}

#[function_component]
fn Body() -> Html {
	let autosync_status = use_context::<storage::autosync::Status>().unwrap();
	let display_route = autosync_status.is_active().then_some("d-none");
	html! {<>
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
							<auth::LoginButton />
						</ul>
					</div>
				</div>
			</nav>
		</header>
		<AutosyncTakeover />
		<div class={classes!(display_route)}>
			<Switch<Route> render={Route::switch} />
		</div>
	</>}
}

#[function_component]
pub fn AutosyncTakeover() -> Html {
	use storage::autosync;
	let autosync_channel = use_context::<autosync::Channel>().unwrap();
	let autosync_status = use_context::<autosync::Status>().unwrap();
	auth::use_on_auth_success(move |_auth_status| {
		autosync_channel.try_send_req(autosync::Request::UpdateLists);
	});

	html! {
		<div class={classes!(
			"sync-status",
			"d-flex", "justify-content-center", "align-items-center",
			(!autosync_status.is_active()).then_some("d-none"),
		)}>
			<div class="d-flex flex-column align-items-center" style="width: 1000px;">
				{autosync_status.stages().iter().enumerate().map(|(idx, stage)| {
					html! {
						<div class="w-100 my-2">
							<div class="d-flex align-items-center">
								{stage.progress.is_none().then(|| {
									html!(<div class="spinner-border me-2" role="status" />)
								})}
								<div class={format!("h{}", idx+1)}>{&stage.title}</div>
							</div>
							{stage.progress.as_ref().map(|status| {
								let progress = (status.progress as f64 / status.max as f64) * 100f64;
								html! {
									<div>
										<div class="progress" role="progressbar">
											<div class="progress-bar bg-success" style={format!("width: {progress}%")} />
										</div>
										<div class="progress-label-float">
											{status.progress} {"/"} {status.max}
										</div>
									</div>
								}
							})}
						</div>
					}
				}).collect::<Vec<_>>()}
			</div>
		</div>
	}
}
