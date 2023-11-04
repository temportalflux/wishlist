use crate::database::ListId;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::use_store_value;

mod auth;
mod data;
mod database;
mod logging;
mod page;
mod spinner;
mod storage;
mod theme;
mod util;

#[cfg(target_family = "wasm")]
fn main() {
	logging::init(logging::Config::default().prefer_target());
	yew::Renderer::<App>::new().render();
}

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
	#[at("/")]
	Collection,
	#[at("/:owner/:id")]
	List { owner: String, id: String },
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
			Self::NotFound => html!(<page::NotFound />),
			Self::Collection => html!(<page::list::Collection />),
			Self::List { owner, id } => {
				let value = ListId { owner, id };
				html!(<page::list::List {value} />)
			}
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct GeneralProp<T: Clone + PartialEq> {
	pub value: T,
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
	let auth_status = use_store_value::<netlify_oauth::Status>();
	let autosync_status = use_context::<storage::autosync::Status>().unwrap();
	let display_route = autosync_status.is_active().then_some("d-none");
	html! {<>
		<header>
			<nav class="navbar navbar-expand-lg sticky-top bg-body-tertiary">
				<div class="container-fluid">
					<Link<Route> classes={classes!("navbar-brand")} to={Route::Collection}>{"Wishlist"}</Link<Route>>
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
			{match &*auth_status {
				netlify_oauth::Status::Successful { .. } => {
					html!(<Switch<Route> render={Route::switch} />)
				}
				_ => html!("Need to login - need to make custom page for this"),
			}}

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
