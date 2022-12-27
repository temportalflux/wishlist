use crate::{
	api::github::gist::{self},
	components::{user, AuthSwitch},
	session::{Profile, SessionValue},
};
use ybc::{Button, Container, NavbarDropdown, NavbarItem, Tile};
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
								<Button classes={"is-dark"} onclick={logout}>{"Sign Out"}</Button>
							</NavbarItem>
						</NavbarDropdown>
					})}
					anonymous={(html! {
						<NavbarItem>
							<Button classes={"is-primary is-dark"} onclick={login}>{"Sign In"}</Button>
						</NavbarItem>
					})}
				/>
			</>})}
		/>
		{ <Route as crate::route::Route>::switch() }
	</>}
}

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
	#[at("/")]
	Home,
	#[at("/guide")]
	UserGuide,
	#[not_found]
	#[at("/404")]
	NotFound,
}

impl crate::route::Route for Route {
	fn html(self) -> Html {
		let create_private = Callback::from(|_| {
			wasm_bindgen_futures::spawn_local(async {
				match gist::FetchProfile::get().await {
					Ok(profile) => profile.apply_to_session(),
					Err(err) => log::debug!("{err:?}"),
				}
			});
		});
		let create_wishlist = Callback::from(|_| {
			wasm_bindgen_futures::spawn_local(async {
				let list = gist::List::new("Test List");
				let mut gist = list.into_gist();
				let result = gist.save().await;
				log::debug!("{result:?}");
				if result.is_ok() {
					let mut profile = Profile::load().unwrap();
					profile.lists.push(gist.id.unwrap());
					profile.apply_to_session();
				}
			});
		});
		match self {
			Self::Home => html! {
				<Container fluid=true>
					<Tile>
						<Tile vertical=true size={ybc::TileSize::Four}>
							<Tile classes={"box"}>
								<Button onclick={create_private}>{"Fetch gist data"}</Button>
								<Button onclick={create_wishlist}>{"Create New Wishlist"}</Button>
							</Tile>
							/* .. snip .. more tiles here .. */
						</Tile>
					</Tile>
				</Container>
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
