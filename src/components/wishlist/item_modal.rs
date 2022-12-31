use crate::components::wishlist::item::{self, Item};
use wasm_bindgen::UnwrapThrowExt;
use ybc::{Button, Control, Field, Icon, Input, InputType, Select, Tag, Tags, TextArea, Title, Section, Container, Level, LevelLeft, LevelRight};
use yew::prelude::*;

use super::item::Kind;

#[derive(Properties, PartialEq)]
pub struct ItemModalProps {
	pub item: Option<(Option<usize>, Item)>,
	pub on_close: Callback<Option<(Option<usize>, Item)>>,
}

#[function_component]
pub fn ItemModal(props: &ItemModalProps) -> Html {
	let (should_show, idx, item) = match &props.item {
		Some((idx, item)) => (true, idx.clone(), item.clone()),
		None => (false, None, Default::default()),
	};
	let state = use_state_eq(|| Item::default());
	let is_shown = use_state_eq(|| false);
	let reset_state = should_show != *is_shown;
	if reset_state {
		is_shown.set(should_show);
		state.set(item.clone());
	}
	if !*is_shown {
		return html! {
			<div class={"modal"} id="wishlist::ItemModal" />
		};
	}

	let discard = {
		let on_close = props.on_close.clone();
		Callback::from(move |_| {
			on_close.emit(None);
		})
	};
	let save_and_close = {
		let idx = idx.clone();
		let state = state.clone();
		let on_close = props.on_close.clone();
		Callback::from(move |_| {
			on_close.emit(Some((idx, (*state).clone())));
		})
	};

	let mutate_item = {
		let state = state.clone();
		Callback::from(move |apply_to_item: Box<dyn FnOnce(&mut Item)>| {
			let mut item = (*state).clone();
			apply_to_item(&mut item);
			state.set(item);
		})
	};

	html! {
		<div class={"modal is-active"} id="wishlist::ItemModal">
			<div class="modal-background" />
			<div class="modal-content">
				<ybc::Box>
					<Title>{"Item"}</Title>
					<ItemFields {reset_state} item={(*state).clone()} mutate_item={mutate_item} can_bundle=true />
					<Field grouped=true grouped_align={ybc::GroupedAlign::Right}>
						<Control><Button classes={"is-danger is-light"} onclick={discard}>{"Discard Changes"}</Button></Control>
						<Control><Button classes={"is-primary"} onclick={save_and_close}>{"Save"}</Button></Control>
					</Field>
				</ybc::Box>
			</div>
		</div>
	}
}

#[derive(Properties, PartialEq)]
struct ItemFieldsProps {
	item: Item,
	reset_state: bool,
	mutate_item: Callback<Box<dyn FnOnce(&mut Item)>>,
	can_bundle: bool,
}

#[function_component]
fn ItemFields(ItemFieldsProps {
	item,
	reset_state,
	mutate_item,
	can_bundle,
}: &ItemFieldsProps) -> Html {
	let custom_tag_text = use_state_eq(|| String::new());
	if *reset_state {
		custom_tag_text.set(String::new());
	}
	let state = item;

	fn reducer<T: 'static>(mutate: &Callback<Box<dyn FnOnce(&mut Item)>>, apply: impl Fn((&mut Item, T)) + 'static) -> Callback<T> {
		let mutate = mutate.clone();
		let apply = std::rc::Rc::new(apply);
		Callback::from(move |value| {
			let apply = apply.clone();
			mutate.emit(Box::new(move |item| apply((item, value))))
		})
	}

	let add_custom_tag = {
		let custom_tag_text = custom_tag_text.clone();
		let apply_tag = reducer(&mutate_item, |(item, new_tag)| { item.tags.insert(new_tag); });
		Callback::from(move |_| {
			let new_tag = (*custom_tag_text).clone();
			if !new_tag.is_empty() {
				custom_tag_text.set(String::new());
				apply_tag.emit(new_tag);
			}
		})
	};

	let kind_section = match &state.kind {
		Kind::Specific(_) => html! {},
		Kind::Idea(_) => html! {},
		Kind::Bundle(bundle) => {
			let mut entry_fields = Vec::with_capacity(bundle.entries.len());
			// This gets real wild because we need to propogate state changes to items in a bundle back to the original item.
			for (entry_idx, entry) in bundle.entries.iter().enumerate() {
				// Create a clone of the callback through which we can mutate the parent item.
				let mutate_parent = mutate_item.clone();
				// Create a callback with the same signature as the general mutator, but which mutates an item within the bundle.
				let mutate_bundle_item = Callback::from(move |apply_to_bundled: Box<dyn FnOnce(&mut Item)>| {
					// Execute the callback to apply some change to the parent item.
					mutate_parent.emit(Box::new(move |parent| {
						// Now that we have the parent item, it is expected to have the bundle type
						// (otherwise this section would never have been generated).
						if let Kind::Bundle(bundle) = &mut parent.kind {
							// So we can get the specific bundled item from the parent.
							let bundled_item = bundle.entries.get_mut(entry_idx).unwrap();
							// And apply the change to that bundled item, thereby mutating the parent item.
							apply_to_bundled(bundled_item);
						}
						// When mutate_parent finishes, it will apply the state, thereby saving our changes.
					}));
				});
				// We can now use the bundle mutator in the sub-items fields.
				entry_fields.push(html! {
					<Container>
						<Level>
							<LevelLeft>
								<Title>{format!("Item #{}", entry_idx + 1)}</Title>
							</LevelLeft>
							<LevelRight>
								<Button>{"Delete Bundled Item"}</Button>
							</LevelRight>
						</Level>
						<ItemFields {reset_state} item={entry.clone()} mutate_item={mutate_bundle_item} can_bundle=false />
					</Container>
				});
			}
			html! {<>
				{entry_fields}
				<Field grouped=true grouped_align={ybc::GroupedAlign::Right}>
					<Control><Button classes={"is-primary"} onclick={{
						reducer(&mutate_item, |(item, _)| {
							if let Kind::Bundle(bundle) = &mut item.kind {
								bundle.entries.push(Item::default());
							}
						})
					}}>{"Add New Item in Bundle"}</Button></Control>
				</Field>
			</>}
		},
	};

	let kind_options = {
		let mut options = Vec::new();
		for kind in item::KindName::all() {
			if *kind == item::KindName::Bundle && !can_bundle {
				continue;
			}
			options.push(html! {
				<option value={kind.value().to_owned()} selected={state.kind.name() == *kind}>{kind.value()}</option>
			});
		}
		options
	};

	html! {<>
		<Field label="Name">
			<Control>
				<Input
					name="name" value={state.name.clone()}
					update={reducer(&mutate_item, |(item, value)| item.name = value)}
					r#type={InputType::Text}
					placeholder={"Ex: Lego set #152"}
				/>
			</Control>
		</Field>
		<Field label="Description">
			<TextArea
				name="description" value={state.description.clone()}
				update={reducer(&mutate_item, |(item, value)| item.description = value)}
			/>
		</Field>
		<Control><label class="label">{"Quantity"}</label></Control>
		<Field addons=true>
			<Control>
				<Button onclick={reducer(&mutate_item, Item::dec_quantity)}>
					<Icon size={ybc::Size::Small}><i class="fas fa-minus" /></Icon>
				</Button>
			</Control>
			<Control>
				<Input
					name="quantity" value={format!("{}", state.quantity)}
					update={reducer(&mutate_item, Item::set_quantity_from_text)}
					r#type={InputType::Text}
					placeholder={"42"}
				/>
			</Control>
			<Control>
				<Button onclick={reducer(&mutate_item, Item::inc_quantity)}>
					<Icon size={ybc::Size::Small}><i class="fas fa-plus" /></Icon>
				</Button>
			</Control>
		</Field>
		<Control><label class="help">{"How many can be reserved?"}</label></Control>
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
			{state.tags.iter().map(|tag| {
				html! {
					<Control>
						<Tags has_addons=true>
							<Tag>{tag}</Tag>
							<Tag classes={"is-delete"} onclick={{
								let tag = tag.clone();
								reducer(&mutate_item, move |(item, _)| { item.tags.remove(&tag); })
							}} />
						</Tags>
					</Control>
				}
			}).collect::<Vec<_>>()}
		</Field>
		<Field label={"Kind"} help={"What type of gift idea is this?"}>
			<Control>
				<Select name="kind" value={state.kind.name().value().to_owned()}
					update={reducer(&mutate_item, |(item, value): (&mut Item, String)| item.kind = item::KindName::from(&value).into())}
				>
					{kind_options}
				</Select>
			</Control>
		</Field>
		{kind_section}
	</>}
}
