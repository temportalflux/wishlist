use crate::{
	api::github::gist::{self, GistId},
	components::wishlist::item::{self, Item, ItemContainer},
	hooks::use_async,
};
use std::collections::{HashMap, VecDeque};
use wasm_bindgen::UnwrapThrowExt;
use ybc::{
	Button, CardContent, CardFooter, Column, Columns, Container, Content, Control,
	Field, Icon, Input, InputType, Level, LevelItem, LevelLeft, LevelRight,
	Notification, Section, Select, Size, Subtitle, Tag, Tags, TextArea, Title,
};
use yew::prelude::*;
use yew_hooks::{use_clipboard, use_drag_with_options, use_drop_with_options, UseDragOptions, UseDropOptions};

#[derive(Clone, PartialEq)]
pub struct Mutator(Callback<Box<dyn FnOnce(&mut Item)>>);
impl Mutator {
	pub fn new<T: 'static + Clone>(
		state: &UseStateHandle<T>,
		get_item: impl Fn(&mut T) -> Option<&mut Item> + 'static,
	) -> Self {
		let state = state.clone();
		Self(Callback::from(
			move |apply_to_item: Box<dyn FnOnce(&mut Item)>| {
				let mut inner = (*state).clone();
				if let Some(item) = get_item(&mut inner) {
					apply_to_item(item);
				}
				state.set(inner);
			},
		))
	}

	pub fn reduce<T: 'static>(&self, apply: impl Fn(&mut Item, T) + 'static) -> Callback<T> {
		let mutator = self.0.clone();
		let apply = std::rc::Rc::new(apply);
		Callback::from(move |value| {
			let apply = apply.clone();
			mutator.emit(Box::new(move |item| apply(item, value)));
		})
	}
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct PageProps {
	pub gist_id: GistId,
}

#[function_component]
pub fn Page(props: &PageProps) -> Html {
	let clipboard = use_clipboard();
	let state = use_state(|| gist::Gist::<gist::List>::default());
	let saved = use_state(|| gist::Gist::<gist::List>::default());
	let item_path = use_state(|| VecDeque::<usize>::new());
	let tags_changed = use_state(|| false);

	let showing_wishlist_settings = use_state(|| false);
	let show_settings = {
		let state = showing_wishlist_settings.clone();
		Callback::from(move |_| {
			state.set(!*state);
		})
	};

	let fetch = {
		let handle = (state.clone(), saved.clone());
		let fetch_id = props.gist_id.clone();
		let fetch = use_async(false, move || {
			let (state, saved) = handle.clone();
			let gist_id = fetch_id.clone();
			Box::pin(async move {
				let gist = gist::Gist::<gist::List>::fetch(gist_id).await?;
				state.set(gist.clone());
				saved.set(gist);
				Ok(()) as anyhow::Result<()>
			})
		});
		// If there is no fetch in progress and we need to fetch, then run the async op.
		if !fetch.loading {
			let should_fetch = match (&state.id, &props.gist_id) {
				(None, _) => true,
				(Some(loaded), desired) if loaded != desired => true,
				_ => false,
			};
			if should_fetch {
				fetch.run();
				state.set(gist::Gist::default());
				saved.set(gist::Gist::default());
			}
		}
		fetch
	};
	let save = {
		let handle = (state.clone(), saved.clone());
		use_async(false, move || {
			let (state, saved) = handle.clone();
			Box::pin(async move {
				let mut gist = (*state).clone();
				gist.save().await?;
				state.set(gist.clone());
				saved.set(gist);
				Ok(()) as anyhow::Result<()>
			})
		})
	};
	if state.id.is_none() || fetch.loading {
		return html! {
			<Container>
				<ybc::Box>
					<Icon size={ybc::Size::Large}>
						<i class="fas fa-circle-notch fa-spin" />
					</Icon>
					<span>{"Fetching wishlist"}</span>
				</ybc::Box>
			</Container>
		};
	}

	if *tags_changed {
		let mut gist = (*state).clone();
		gist.file.rebuild_tags();
		state.set(gist);
		tags_changed.set(false);
	}

	let copy_link_to_clipboard = {
		let clipboard = clipboard.clone();
		let absolute_route = crate::to_abs_route(props.gist_id.as_route());
		Callback::from(move |_| {
			clipboard.write_text(absolute_route.clone());
		})
	};

	let has_changes = *state != *saved;
	let changes_notification = {
		let mut classes = classes! {"is-success", "is-light", "px-3", "py-2"};
		if !has_changes {
			classes.push("is-hidden");
		}
		let discard_changes = {
			let saved = saved.clone();
			let state = state.clone();
			Callback::from(move |_| {
				state.set((*saved).clone());
			})
		};
		let save_changes = {
			let save_async = save.trigger().clone();
			Callback::from(move |_| {
				save_async();
			})
		};
		let content = match (has_changes, save.loading) {
			(_, true) => html! {<>
				<Icon size={ybc::Size::Large}>
					<i class="fas fa-circle-notch fa-spin" />
				</Icon>
				<span>{"Saving Changes"}</span>
			</>},
			_ => html! {<>
				<span class="m-1">{"You have unsaved changes to your wishlist!"}</span>
				<Button classes={"is-primary mx-2"} onclick={save_changes}>{"Save Changes"}</Button>
				<Button classes={"is-warning mx-2"} onclick={discard_changes}>{"Discard Changes"}</Button>
			</>},
		};
		html! {
			<Notification {classes}>
				<Content classes="has-text-centered is-align-items-center is-flex">
					{content}
				</Content>
			</Notification>
		}
	};

	let delete_wishlist = Callback::from(|_| {
		log::debug!("TODO: Prompt delete modal");
	});

	let create_item = {
		let item_path = item_path.clone();
		let state = state.clone();
		Callback::from(move |_| {
			let idx = {
				let mut gist = (*state).clone();
				let idx = gist.file.items.len();
				gist.file.items.push(Item::default());
				state.set(gist);
				idx
			};
			item_path.set(VecDeque::from([idx]));
		})
	};

	let tag_filters = match state.file.all_item_tags.is_empty() {
		true => html! {},
		false => {
			html! {<>
				<Tag classes={"is-rounded is-info"}>{"All"}</Tag>
				{state.file.all_item_tags.iter().map(|tag| html! {
					<Tag classes={"is-rounded"} onclick={Callback::from(|_| {
						log::debug!("TODO: tag filers");
					})}>{tag}</Tag>
				}).collect::<Vec<_>>()}
			</>}
		}
	};

	fn get_current_item<'a>(
		gist: &'a mut gist::Gist<gist::List>,
		item_path: &UseStateHandle<VecDeque<usize>>,
	) -> Option<&'a mut Item> {
		gist.file.get_item_mut_at((**item_path).clone())
	}

	let mutator = Mutator::new(&state, {
		let item_path = item_path.clone();
		move |state| get_current_item(state, &item_path)
	});

	let set_current_path = {
		let item_path = item_path.clone();
		Callback::from(move |path| item_path.set(path))
	};

	let item_cards = html! {<>
		{state.file.items.iter().enumerate().map(|(idx, item)| html! {
			<ItemCard item={item.clone()} path_to_item={VecDeque::from([idx])}
				on_edit={set_current_path.clone()}
				on_delete={{
					let state = state.clone();
					Callback::from(move |_| {
						// TODO: Prompt delete modal
						let mut gist = (*state).clone();
						gist.file.remove_item(idx);
						state.set(gist);
					})
				}}
			/>
		}).collect::<Vec<_>>()}
		<ItemCardBase can_drag=false can_drop_onto=true>
			<CardContent>
				{"End of List"}
			</CardContent>
		</ItemCardBase>
	</>};

	let tag_header = html! {<>
		<Columns>
			<Column classes="is-2">
				<Content>
					<Button classes={"is-primary is-fullwidth"} onclick={create_item}>
						<Icon size={Size::Small}><i class="fas fa-plus" /></Icon>
						<span>{"New Item"}</span>
					</Button>
				</Content>
			</Column>
			<Column>
				<Tags classes={"is-justify-content-center are-medium"}>{tag_filters}</Tags>
			</Column>
		</Columns>
	</>};

	let page_content = match *showing_wishlist_settings {
		true => html! {
			<Button classes={"is-danger is-light"} onclick={delete_wishlist.clone()}>
				<Icon size={Size::Small}><i class="fas fa-trash" /></Icon>
				<span>{"Permanently Delete Wishlist"}</span>
			</Button>
		},
		false => {
			let path_indices = (*item_path).clone();
			let path_count = path_indices.len();
			let path_segments = {
				let set_current_path = set_current_path.clone();
				state
					.file
					.get_items_for(path_indices)
					.into_iter()
					.enumerate()
					.map(move |(idx, (path_to_item, item))| {
						let mut item_classes = classes! {};
						if idx == path_count - 1 {
							item_classes.push("is-active");
						}
						let onclick = set_current_path.reform(move |_| path_to_item.clone());
						html! { <li class={item_classes}><a {onclick}>{&item.name}</a></li> }
					})
					.collect::<Vec<_>>()
			};
			let item = get_current_item(&mut (*state).clone(), &item_path).cloned();
			match item {
				Some(item) => html! {<>
					<Level>
						<LevelLeft>
							<LevelItem>
								<span style={"padding-right: 5px;"}>{"Item Path:"}</span>
								<ybc::Breadcrumb>{path_segments}</ybc::Breadcrumb>
							</LevelItem>
						</LevelLeft>
						<LevelRight>
							<LevelItem>
								<Button onclick={{
									let item_path = item_path.clone();
									Callback::from(move |_| {
										let mut prev = (*item_path).clone();
										prev.pop_back();
										item_path.set(prev);
									})
								}}>{"Back"}</Button>
							</LevelItem>
						</LevelRight>
					</Level>
					<ItemData {item} {mutator}
						path_to_item={(*item_path).clone()}
						on_set_path={set_current_path.clone()}
						on_tags_changed={{
							let tags_changed = tags_changed.clone();
							Callback::from(move |_| tags_changed.set(true))
						}}
						can_bundle={item_path.len() <= 1}
					/>
				</>},

				None => html! {<>
					{tag_header}
					{item_cards}
				</>},
			}
		}
	};

	html! {<>
		<Section>
			<Level>
				<LevelLeft>
					<LevelItem>
						<div>
							<Title>
								{saved.file.name.clone()}
								{saved.visibility.as_tag(Some(classes!{"ml-2"}))}
								<Tag tag="a" classes={"ml-2"} onclick={copy_link_to_clipboard.clone()}>{"Copy Link"}</Tag>
							</Title>
							<Subtitle classes={"mt-0"}>{"Owned By: "}{saved.owner_login.clone()}</Subtitle>
						</div>
					</LevelItem>
				</LevelLeft>
				<LevelRight>
					<LevelItem>
						<Button classes={"is-link is-light"} onclick={show_settings}>
							<Icon size={Size::Small}><i class="fas fa-gear" /></Icon>
							<span>{match *showing_wishlist_settings {
								true => "Back",
								false => "Actions",
							}}</span>
						</Button>
					</LevelItem>
				</LevelRight>
			</Level>
			{changes_notification}
			{page_content}
		</Section>
	</>}
}

#[derive(Properties, PartialEq)]
pub struct ItemCardBaseProps {
	#[prop_or_default]
	pub id: Option<String>,
	#[prop_or_default]
	pub can_drag: bool,
	#[prop_or_default]
	pub can_drop_onto: bool,
	#[prop_or_default]
	pub children: Children,
}

#[function_component]
pub fn ItemCardBase(ItemCardBaseProps {
	id, can_drag, can_drop_onto, children
}: &ItemCardBaseProps) -> Html {
	let node = use_node_ref();
	let drop_over_with_valid_src = use_state(|| false);

	let drag_id = id.clone().unwrap_or_default();
	let _drag = use_drag_with_options(
		node.clone(),
		UseDragOptions {
			ondragstart: can_drag.then(move || Box::new(move |e: DragEvent| {
				let Some(data_transfer) = e.data_transfer() else { return; };
				let _ = data_transfer.clear_data();
				let _ = data_transfer.set_data("text/plain", &format!("item:{drag_id}"));
			}) as Box<dyn FnMut(DragEvent) + 'static>),
			..Default::default()
		},
	);

	let drop_id = id.clone().unwrap_or_default();
	let drop = use_drop_with_options(
		node.clone(),
		UseDropOptions {
			ondrop: can_drop_onto.then(move || Box::new(move |e: DragEvent| {
				let Some(data_transfer) = e.data_transfer() else { return; };
				let Ok(data) = data_transfer.get_data("text") else { return; };
				if data.starts_with("item:") {
					let Some(dragged_id) = data.split(':').skip(1).next() else { return; };
					log::debug!("Dropped item {dragged_id:?} onto item {drop_id:?}");
				}
			}) as Box<dyn FnMut(DragEvent) + 'static>),
			..Default::default()
		},
	);

	log::debug!("{:?}", *drop.text);

	let style = {
		let mut attrs = HashMap::new();
		if *drop_over_with_valid_src {
			attrs.insert("border-style", "dashed".to_owned());
		}
		attrs
			.into_iter()
			.map(|(k, v)| format!("{k}:{v};"))
			.collect::<Vec<_>>()
			.join("")
	};
	
	html! {
		<div class={"card block"} ref={node} {style}>
			{children.clone()}
		</div>
	}
}

#[derive(Properties, PartialEq)]
pub struct ItemCardProps {
	pub item: Item,
	pub path_to_item: VecDeque<usize>,
	pub on_edit: Callback<VecDeque<usize>>,
	pub on_delete: Callback<()>,
}

#[function_component]
pub fn ItemCard(props: &ItemCardProps) -> Html {
	let item_img = match &props.item.kind {
		item::Kind::Specific(item) => match &item.image_url {
			Some(url) => Some(html! {
				<figure class={"image is-128x128"} style="margin: auto;">
					<img src={url.clone()} style="max-height: 100%; width: auto; margin: auto;" />
				</figure>
			}),
			None => None,
		},
		item::Kind::Idea(item)  => match &item.image_url {
			Some(url) => Some(html! {
				<figure class={"image is-128x128"} style="margin: auto;">
					<img src={url.clone()} style="max-height: 100%; width: auto; margin: auto;" />
				</figure>
			}),
			None => None,
		},
		// TODO: Img display set
		item::Kind::Bundle(_) => None,
	};
	
	// TODO: Quantity Requested (only for owners)
	// TODO: Quantity Available for reservation (later, and only for non-owners)

	let card_style = {
		let mut attrs = HashMap::new();
		attrs.insert("background-color", props.item.kind.name().color());
		attrs
			.into_iter()
			.map(|(k, v)| format!("{k}:{v};"))
			.collect::<Vec<_>>()
			.join("")
	};
	html! {
		<ItemCardBase id={props.path_to_item.back().map(usize::to_string)} can_drag=true can_drop_onto=true>
			<header class="card-header" style={card_style}>
				<p class="card-header-title">
					{props.item.kind.name().value()}
					{":  "}
					{&props.item.name}
				</p>
				<Button
					classes={"card-header-icon has-text-danger"}
					onclick={props.on_delete.reform(|_| {})}
				><Icon size={Size::Small}><i class="fas fa-trash" /></Icon></Button>
			</header>
			<CardContent>
				<Columns>
					{item_img.map(|img| html! {
						<Column classes="is-2">
							{img}
						</Column>
					}).unwrap_or(html! {})}
					<Column>
						<p>
							<span class="label" style="display: inline; padding-right: 5px;">{"Cost:"}</span>
							{"$"}{props.item.cost()}
						</p>
						{match props.item.description.is_empty() {
							true => html! {},
							false => html! {
								<Content>
									<span class="label">{"Description:"}</span>
									<pre style="background-color: transparent; padding: 0px 0px 0px 10px;">{&props.item.description}</pre>
								</Content>
							},
						}}
					</Column>
				</Columns>
			</CardContent>
			<CardFooter>
				<Button classes={"card-footer-item"} onclick={{
					let path_to_item = props.path_to_item.clone();
					props.on_edit.reform(move |_| path_to_item.clone())
				}}>{"Edit"}</Button>
			</CardFooter>
		</ItemCardBase>
	}
}

#[derive(Properties, PartialEq)]
pub struct ItemDataProps {
	item: Item,
	mutator: Mutator,
	path_to_item: VecDeque<usize>,
	can_bundle: bool,
	on_set_path: Callback<VecDeque<usize>>,
	on_tags_changed: Callback<()>,
}

#[function_component]
pub fn ItemData(
	ItemDataProps {
		item,
		mutator,
		path_to_item,
		can_bundle,
		on_set_path,
		on_tags_changed,
	}: &ItemDataProps,
) -> Html {
	let custom_tag_text = use_state_eq(|| String::new());

	let add_custom_tag = {
		let custom_tag_text = custom_tag_text.clone();
		let on_tags_changed = on_tags_changed.clone();
		let apply_tag = mutator.reduce(move |item, new_tag| {
			item.tags.insert(new_tag);
		});
		Callback::from(move |_| {
			let new_tag = (*custom_tag_text).clone();
			if !new_tag.is_empty() {
				custom_tag_text.set(String::new());
				apply_tag.emit(new_tag);
				on_tags_changed.emit(());
			}
		})
	};

	let tag_elements = match can_bundle {
		false => html! {},
		true => html! {<>
			<Control><label class="label">{"Tags"}</label></Control>
			<Field addons=true>
				<Control>
					<input
						class="input is-small"
						name="custom_tag"
						value={(*custom_tag_text).clone()}
						oninput={{
							let custom_tag_text = custom_tag_text.clone();
							Callback::from(move |ev: web_sys::InputEvent| {
								let input: web_sys::HtmlInputElement = ev.target_dyn_into().expect_throw("event target should be an input");
								custom_tag_text.set(input.value());
							})
						}}
						onkeypress={{
							let add_custom_tag = add_custom_tag.clone();
							Callback::from(move |ev: web_sys::KeyboardEvent| {
								if ev.key() == "Enter" {
									add_custom_tag.emit(());
								}
							})
						}}
					/>
				</Control>
				<Control>
					<Button classes={"is-small"} onclick={add_custom_tag.reform(|_| {})}>
						<Icon size={ybc::Size::Small}><i class="fas fa-plus" /></Icon>
						<span>{"Add Tag"}</span>
					</Button>
				</Control>
			</Field>
			<Field grouped=true multiline=true>
				{item.tags.iter().map(|tag| {
					html! {
						<Control>
							<Tags has_addons=true>
								<Tag>{tag}</Tag>
								<Tag classes={"is-delete"} onclick={{
									let tag = tag.clone();
									let on_tags_changed = on_tags_changed.clone();
									let rm_tag = mutator.reduce(move |item, _| {
										item.tags.remove(&tag);
									});
									Callback::from(move |_| {
										rm_tag.emit(());
										on_tags_changed.emit(());
									})
								}} />
							</Tags>
						</Control>
					}
				}).collect::<Vec<_>>()}
			</Field>
		</>},
	};

	let kind_options = {
		let mut options = Vec::new();
		for kind in item::KindName::all() {
			if *kind == item::KindName::Bundle && !can_bundle {
				continue;
			}
			options.push(html! {
				<option value={kind.value().to_owned()} selected={item.kind.name() == *kind}>{kind.value()}</option>
			});
		}
		options
	};

	let kind_section = match &item.kind {
		item::Kind::Specific(item) => {
			html! {<>
				<Field label="Image">
					<Control>
						<Input
							name="image_url" value={item.image_url.clone().unwrap_or_default()}
							update={mutator.reduce(|item, value: String| {
								let item::Kind::Specific(item) = &mut item.kind else { return; };
								item.image_url = match value.is_empty() {
									false => Some(value),
									true => None,
								};
							})}
							r#type={InputType::Text}
						/>
					</Control>
				</Field>
				<Field label="Offer">
					<Control>
						<Input
							name="offer_url" value={item.offer_url.clone()}
							update={mutator.reduce(|item, value: String| {
								let item::Kind::Specific(item) = &mut item.kind else { return; };
								item.offer_url = value;
							})}
							r#type={InputType::Text}
						/>
					</Control>
				</Field>
				<Field label="Cost (Per Unit)">
					<Control>
						<Input
							name="cost_per_unit" value={item.cost_per_unit_str.clone()}
							classes={{
								let mut classes = classes!{};
								if item.cost_per_unit_str.parse::<f32>().is_err() {
									classes.push("is-danger");
								}
								classes
							}}
							update={mutator.reduce(|item, value: String| {
								let item::Kind::Specific(item) = &mut item.kind else { return; };
								if let Ok(cost) = value.parse::<f32>() {
									item.cost_per_unit = cost;
								}
								item.cost_per_unit_str = value;
							})}
							r#type={InputType::Text}
						/>
					</Control>
				</Field>
			</>}
		}
		item::Kind::Idea(item) => html! {<>
			<Field label="Image">
				<Control>
					<Input
						name="image_url" value={item.image_url.clone().unwrap_or_default()}
						update={mutator.reduce(|item, value: String| {
							let item::Kind::Idea(item) = &mut item.kind else { return; };
							item.image_url = match value.is_empty() {
								false => Some(value),
								true => None,
							};
						})}
						r#type={InputType::Text}
					/>
				</Control>
			</Field>
			<Field label="Estimated Cost (Per Unit)">
				<Control>
					<Input
						name="estimated_cost" value={item.estimated_cost_str.clone()}
						classes={{
							let mut classes = classes!{};
							if item.estimated_cost_str.parse::<f32>().is_err() {
								classes.push("is-danger");
							}
							classes
						}}
						update={mutator.reduce(|item, value: String| {
							let item::Kind::Idea(item) = &mut item.kind else { return; };
							if let Ok(cost) = value.parse::<f32>() {
								item.estimated_cost = cost;
							}
							item.estimated_cost_str = value;
						})}
						r#type={InputType::Text}
					/>
				</Control>
			</Field>
			<Control><label class="label">{"Example Offers"}</label></Control>
			<Control>
				<Button classes={"is-primary"} onclick={mutator.reduce(|item, _| {
					let item::Kind::Idea(item) = &mut item.kind else { return; };
					item.example_urls.push(String::new());
				})}>
					<Icon size={ybc::Size::Small}><i class="fas fa-plus" /></Icon>
					<span>{"Add Example"}</span>
				</Button>
			</Control>
			{item.example_urls.iter().enumerate().map(|(idx, value)| html! {
				<Field addons=true>
					<Control>
						<Button classes={"has-text-danger"} onclick={mutator.reduce(move |item, _| {
							let item::Kind::Idea(item) = &mut item.kind else { return; };
							item.example_urls.remove(idx);
						})}>
							<Icon size={ybc::Size::Small}><i class="fas fa-minus" /></Icon>
						</Button>
					</Control>
					<Control classes={"is-flex-grow-1"}>
						<Input
							name={format!("{idx}")} value={value.clone()}
							update={mutator.reduce(move |item, value: String| {
								let item::Kind::Idea(item) = &mut item.kind else { return; };
								*item.example_urls.get_mut(idx).unwrap() = value;
							})}
							r#type={InputType::Text}
						/>
					</Control>
				</Field>
			}).collect::<Vec<_>>()}
		</>},
		item::Kind::Bundle(bundle) => {
			html! {<>
				<div class={"block"}>
					{bundle.entries.iter().enumerate().map(|(idx, item)| html! {
						<ItemCard
							item={item.clone()}
							path_to_item={{
								let mut path = path_to_item.clone();
								path.push_back(idx);
								path
							}}
							on_edit={on_set_path}
							on_delete={mutator.reduce(move |item, _| item.remove_item(idx))}
						/>
					}).collect::<Vec<_>>()}
				</div>
			</>}
		}
	};

	html! {<>
		<Field label="Name">
			<Control>
				<Input
					name="name" value={item.name.clone()}
					update={mutator.reduce(|item, value| item.name = value)}
					r#type={InputType::Text}
					placeholder={"Ex: Lego set #152"}
				/>
			</Control>
		</Field>
		<Field label="Description">
			<TextArea
				name="description" value={item.description.clone()}
				update={mutator.reduce(|item, value| item.description = value)}
			/>
		</Field>
		<Control><label class="label">{"Quantity"}</label></Control>
		<Field addons=true>
			<Control>
				<Button onclick={mutator.reduce(Item::dec_quantity)}>
					<Icon size={ybc::Size::Small}><i class="fas fa-minus" /></Icon>
				</Button>
			</Control>
			<Control>
				<Input
					name="quantity" value={format!("{}", item.quantity)}
					update={mutator.reduce(Item::set_quantity_from_text)}
					r#type={InputType::Text}
					placeholder={"42"}
				/>
			</Control>
			<Control>
				<Button onclick={mutator.reduce(Item::inc_quantity)}>
					<Icon size={ybc::Size::Small}><i class="fas fa-plus" /></Icon>
				</Button>
			</Control>
		</Field>
		<Control><label class="help">{"How many can be reserved?"}</label></Control>
		{tag_elements}
		<Field label={"Kind"} help={"What type of gift idea is this?"}>
			<Control>
				<Select name="kind" value={item.kind.name().value().to_owned()}
					update={mutator.reduce(|item, value: String| item.kind = item::KindName::from(&value).into())}
				>
					{kind_options}
				</Select>
			</Control>
		</Field>
		{kind_section}
	</>}
}
