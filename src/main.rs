use ybc::{Button, Image, NavbarDropdown, NavbarItem, Container, Tile, NavbarItemTag};
use yew::prelude::*;
use yew_router::{BrowserRouter, Routable, Switch};

enum Action {
	SignIn,
}

struct Index {
	token: bool,
}

impl Component for Index {
	type Message = Action;
	type Properties = ();

	fn create(_ctx: &Context<Self>) -> Self {
		Self { token: false }
	}

	#[allow(unused_parens)]
	fn view(&self, ctx: &Context<Self>) -> Html {
		let link = ctx.link();
		let signin = link.callback(|_| Action::SignIn);

		let account = match self.token {
			true => html! {
				<NavbarDropdown navlink={(html! {<>
					<Image size={Some(ybc::ImageSize::Is32x32)}>
						<img class="is-rounded" src="https://bulma.io/images/placeholders/32x32.png" />
					</Image>
					{"Name"}
				</>})}>
					<NavbarItem href={"/logout"} tag={NavbarItemTag::A}>{"Logout"}</NavbarItem>
				</NavbarDropdown>
			},
			false => html! {
				<NavbarItem>
					<Button classes={"is-primary is-dark"} onclick={signin}>{"Sign In"}</Button>
				</NavbarItem>
			},
		};

		html! {<>
			<ybc::Navbar classes={"is-dark"}
				navbrand={Some(html! {
					<NavbarItem href={"/"} tag={NavbarItemTag::A}>
						<img src="https://bulma.io/images/bulma-logo.png" width="112" height="28" />
					</NavbarItem>
				})}
				navstart={Some(html! {<>
					<NavbarItem href={"/"} tag={NavbarItemTag::A}>{"Home"}</NavbarItem>
					<NavbarItem href={"/guide"} tag={NavbarItemTag::A}>{"User Guide"}</NavbarItem>
				</>})}
				navend={Some(html! {<>
					{account}
				</>})}
			/>
			<BrowserRouter>
				<Switch<Route> render={Switch::render(Route::parse)} />
			</BrowserRouter>
		</>}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Action::SignIn => {
				log::debug!("Sign In");
				self.token = true;
				true
			}
		}
	}
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

impl Route {
	fn parse(route: &Route) -> Html {
		match route {
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

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::start_app::<Index>();
}
