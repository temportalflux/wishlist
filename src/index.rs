use ybc::{Button, Container, Image, NavbarDropdown, NavbarItem, NavbarItemTag, Tile};
use yew::prelude::*;
use yew_oauth2::{
	oauth2::Client,
	prelude::{Authenticated, NotAuthenticated, OAuth2Dispatcher, OAuth2Operations},
};
use yew_router::Routable;

pub struct Page;
impl Component for Page {
	type Message = ();
	type Properties = ();

	fn create(_ctx: &Context<Self>) -> Self {
		Self
	}

	#[allow(unused_parens)]
	fn view(&self, ctx: &Context<Self>) -> Html {
		let login = ctx.link().callback_once(|_| {
			OAuth2Dispatcher::<Client>::new().start_login();
		});
		let logout = ctx.link().callback_once(|_| {
			OAuth2Dispatcher::<Client>::new().logout();
		});

		html! {<>
			<ybc::Navbar classes={"is-dark"}
				navbrand={Some(html! {
					<NavbarItem href={Route::Home} tag={NavbarItemTag::A}>
						<img src="https://bulma.io/images/bulma-logo.png" width="112" height="28" />
					</NavbarItem>
				})}
				navstart={Some(html! {<>
					<NavbarItem href={Route::Home} tag={NavbarItemTag::A}>{"Home"}</NavbarItem>
					<NavbarItem href={Route::UserGuide} tag={NavbarItemTag::A}>{"User Guide"}</NavbarItem>
				</>})}
				navend={Some(html! {<>
					<Authenticated>
						<NavbarDropdown navlink={(html! {<>
							<Image size={Some(ybc::ImageSize::Is32x32)}>
								<img class="is-rounded" src="https://bulma.io/images/placeholders/32x32.png" />
							</Image>
							{"Name"}
						</>})}>
							<NavbarItem>
								<Button classes={"is-dark"} onclick={logout}>{"Sign Out"}</Button>
							</NavbarItem>
						</NavbarDropdown>
					</Authenticated>
					<NotAuthenticated>
						<NavbarItem>
							<Button classes={"is-primary is-dark"} onclick={login}>{"Sign In"}</Button>
						</NavbarItem>
					</NotAuthenticated>
				</>})}
			/>
			{ <Route as crate::route::Route>::switch() }
		</>}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
	#[at("")]
	Home,
	#[at("guide")]
	UserGuide,
	#[not_found]
	#[at("404")]
	NotFound,
}

impl crate::route::Route for Route {
	fn html(&self) -> Html {
		match self {
			Self::Home => html! {
				<Container fluid=true>
					<Tile>
						<Tile vertical=true size={ybc::TileSize::Four}>
							<Tile classes={"box"}>
								<p>{"Lorem ipsum dolor sit amet ..."}</p>
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
