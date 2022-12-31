use crate::components::wishlist::item::{self, Item};
use wasm_bindgen::UnwrapThrowExt;
use ybc::{Button, Control, Field, Icon, Input, InputType, Select, Tag, Tags, TextArea, Title};
use yew::prelude::*;

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
	let custom_tag_text = use_state_eq(|| String::new());
	if should_show != *is_shown {
		is_shown.set(should_show);
		state.set(item);
		custom_tag_text.set(String::new());
	}
	if !*is_shown {
		return html! {
			<div class={"modal"} id="wishlist::ItemModal" />
		};
	}

	fn mutate(state: &UseStateHandle<Item>, apply: impl FnOnce(&mut Item) + 'static) {
		let mut item = (**state).clone();
		apply(&mut item);
		state.set(item);
	}

	fn reducer<T: 'static>(
		state: &UseStateHandle<Item>,
		apply: impl Fn((&mut Item, T)) + 'static,
	) -> Callback<T, ()> {
		let state = state.clone();
		Callback::from(move |value| {
			let mut item = (*state).clone();
			apply((&mut item, value));
			state.set(item);
		})
	}

	let add_custom_tag = {
		let custom_tag_text = custom_tag_text.clone();
		let state = state.clone();
		Callback::from(move |_| {
			let new_tag = (*custom_tag_text).clone();
			custom_tag_text.set(String::new());
			mutate(&state, move |item| {item.tags.insert(new_tag);});
		})
	};

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

	html! {
		<div class={"modal is-active"} id="wishlist::ItemModal">
			<div class="modal-background" />
			<div class="modal-content">
				<ybc::Box>
					<Title>{"Item"}</Title>
					<Field label="Name">
						<Control>
							<Input
								name="name" value={state.name.clone()}
								update={reducer(&state, |(item, value)| item.name = value)}
								r#type={InputType::Text}
								placeholder={"Ex: Lego set #152"}
							/>
						</Control>
					</Field>
					<Field label="Description">
						<TextArea
							name="description" value={state.description.clone()}
							update={reducer(&state, |(item, value)| item.description = value)}
						/>
					</Field>
					<Control><label class="label">{"Quantity"}</label></Control>
					<Field addons=true>
						<Control>
							<Button onclick={reducer(&state, Item::dec_quantity)}>
								<Icon size={ybc::Size::Small}><i class="fas fa-minus" /></Icon>
							</Button>
						</Control>
						<Control>
							<Input
								name="quantity" value={format!("{}", state.quantity)}
								update={reducer(&state, Item::set_quantity_from_text)}
								r#type={InputType::Text}
								placeholder={"42"}
							/>
						</Control>
						<Control>
							<Button onclick={reducer(&state, Item::inc_quantity)}>
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
											reducer(&state, move |(item, _)| { item.tags.remove(&tag); })
										}} />
									</Tags>
								</Control>
							}
						}).collect::<Vec<_>>()}
					</Field>
					<Field label={"Kind"} help={"What type of gift idea is this?"}>
						<Control>
							<Select name="kind" value={state.kind.name().value().to_owned()}
								update={reducer(&state, |(item, value): (&mut Item, String)| item.kind = item::KindName::from(&value).into())}
							>
								{item::KindName::all().iter().map(|kind| html! {
									<option value={kind.value().to_owned()} selected={state.kind.name() == *kind}>{kind.value()}</option>
								}).collect::<Vec<_>>()}
							</Select>
						</Control>
					</Field>
					<Field grouped=true grouped_align={ybc::GroupedAlign::Right}>
						<Control><Button classes={"is-danger is-light"} onclick={discard}>{"Discard Changes"}</Button></Control>
						<Control><Button classes={"is-primary"} onclick={save_and_close}>{"Save"}</Button></Control>
					</Field>
				</ybc::Box>
			</div>
		</div>
	}
}
