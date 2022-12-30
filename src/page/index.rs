use super::Home;
use crate::{
	api::github::gist::GistId,
	components::{user, wishlist, AuthSwitch},
};
use ybc::{Button, Container, Icon, NavbarDropdown, NavbarItem, Tile};
use yew::prelude::*;
use yew_router::{
	prelude::{use_navigator, Link},
	Routable,
};

#[allow(unused_parens)]
#[function_component]
pub fn Page() -> Html {
	let navigator = use_navigator().unwrap();
	let login = {
		let navigator = navigator.clone();
		Callback::from(move |_| {
			navigator.push(&crate::api::auth::Route::Login);
		})
	};
	let logout = {
		let navigator = navigator.clone();
		Callback::from(move |_| {
			navigator.push(&crate::api::auth::Route::Logout);
		})
	};

	html! {<>
		<ybc::Navbar classes={"is-dark"}
			navbrand={Some(html! {
				<Link<Route> classes={"navbar-item"} to={Route::Home}>
					<img src="https://bulma.io/images/bulma-logo.png" width="112" height="28" />
				</Link<Route>>
			})}
			navstart={Some(html! {<>
				<Link<Route> classes={"navbar-item"} to={Route::Home}>{"Home"}</Link<Route>>
				<Link<Route> classes={"navbar-item"} to={Route::UserGuide}>{"User Guide"}</Link<Route>>
			</>})}
			navend={Some(html! {<>
				<AuthSwitch
					identified={(html! {
						<NavbarDropdown navlink={(html! {<>
							<user::Identification />
						</>})}>
							<NavbarItem>
								<Button classes={"is-danger is-light"} onclick={logout}>
									<Icon><i class="fas fa-right-from-bracket" /></Icon>
									<span>{"Sign Out"}</span>
								</Button>
							</NavbarItem>
						</NavbarDropdown>
					})}
					anonymous={(html! {
						<NavbarItem>
							<Button classes={"is-primary is-dark"} onclick={login}>
								<Icon><i class="fab fa-github" /></Icon>
								<span>{"Sign In"}</span>
							</Button>
						</NavbarItem>
					})}
				/>
			</>})}
		/>
		{ <Route as crate::route::Route>::switch() }
	</>}
}

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
	#[at("/")]
	Home,
	#[at("/guide")]
	UserGuide,
	#[at("/:gist_id")]
	List { gist_id: GistId },
	#[not_found]
	#[at("/404")]
	NotFound,
}

impl crate::route::Route for Route {
	fn html(self) -> Html {
		match self {
			Self::Home => html! {
				<Home />
			},
			Self::UserGuide => html! {
				<Container fluid=true>
					<Tile>
						<Tile vertical=true size={ybc::TileSize::Four}>
							<Tile classes={"box"}>
								<p>{"This is the user guide, TBD"}</p>
							</Tile>
							/* .. snip .. more tiles here .. */
						</Tile>
					</Tile>
				</Container>
			},
			Self::List { gist_id } => html! { <wishlist::Page gist_id={gist_id} /> },
			Self::NotFound => html! {
				<h1>{"404: Page not found"}</h1>
			},
		}
	}
}

impl yew::html::IntoPropValue<Option<String>> for Route {
	fn into_prop_value(self) -> Option<String> {
		Some(self.to_path())
	}
}
