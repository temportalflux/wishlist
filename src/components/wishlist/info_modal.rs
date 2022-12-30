use crate::{
	api::github::gist::{self, GistId, GistInfo, Visibility},
	session::Profile,
};
use serde::{Deserialize, Serialize};
use std::{ops::DerefMut, str::FromStr};
use ybc::{Button, Control, Field, Input, InputType, Select};
use yew::prelude::*;
use yewdux::{
	prelude::{use_store, Dispatch},
	store::Store,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "session")]
pub struct InfoModalPrompt(Option<InfoModalProps>);
impl InfoModalPrompt {
	pub fn open(props: InfoModalProps) {
		Dispatch::<Self>::new().set(Self(Some(props)));
	}

	fn props(&self) -> Option<&InfoModalProps> {
		self.0.as_ref()
	}

	pub fn close(&mut self) {
		self.0 = None;
	}
}
impl From<InfoModalProps> for InfoModalPrompt {
	fn from(props: InfoModalProps) -> Self {
		Self(Some(props))
	}
}
impl std::ops::Deref for InfoModalPrompt {
	type Target = InfoModalProps;

	fn deref(&self) -> &Self::Target {
		self.props().unwrap()
	}
}
impl DerefMut for InfoModalPrompt {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0.as_mut().unwrap()
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Properties)]
pub struct InfoModalProps {
	/// The id of the gist the wishlist is stored in
	pub id: Option<GistId>,
	pub title: String,
	pub owner_login: String,
	pub visibility: Visibility,
}
impl InfoModalProps {
	pub fn gist_info(&self) -> Option<GistInfo> {
		match &self.id {
			Some(id) => Some(GistInfo {
				id: id.clone(),
				title: self.title.clone(),
				owner_login: self.owner_login.clone(),
				visibility: self.visibility,
			}),
			None => None,
		}
	}
}

#[function_component]
pub fn InfoModal() -> Html {
	let (prompt, dispatch) = use_store::<InfoModalPrompt>();
	let Some(props) = prompt.props() else {
		return html! {
			<div class={"modal"} id="wishlist::InfoModal" />
		};
	};

	let update_title = dispatch.reduce_mut_callback_with(|prompt, title| prompt.title = title);
	let update_visibility = dispatch.reduce_mut_callback_with(|prompt, vis_str: String| {
		prompt.visibility = Visibility::from_str(&vis_str).unwrap();
	});
	let discard = dispatch.reduce_mut_callback(InfoModalPrompt::close);
	let save_and_close = dispatch.reduce_mut_future_callback(|prompt| {
		Box::pin(async move {
			let list = gist::List::new(prompt.title.clone()).with_visibility(prompt.visibility);
			let mut gist = list.into_gist();
			match gist.save().await {
				Ok(_) => {
					prompt.id = Some(gist.id.unwrap());
					Dispatch::<Profile>::new().reduce_mut(|profile| {
						profile.lists.push(prompt.gist_info().unwrap());
					});
					prompt.close();
				}
				Err(err) => log::error!("Failed to save wishlist: {err:?}"),
			}
		})
	});
	html! {
		<div class={"modal is-active"} id="wishlist::InfoModal">
			<div class="modal-background"></div>
			<div class="modal-content">
				<ybc::Box>
			<Field label={"Title"}>
				<Control>
					<Input
						name="title" value={props.title.clone()}
						update={update_title}
						r#type={InputType::Text}
						placeholder={"Ex: Birthday"}
					/>
				</Control>
			</Field>
			<Field label={"Visibility"} help={"Who else can see this wishlist?"}>
				<Control>
					<Select name="visibility" value={props.visibility} update={update_visibility}>
						<option value={Visibility::Public.value()} selected={props.visibility == Visibility::Public}>{Visibility::Public}</option>
						<option value={Visibility::Private.value()} selected={props.visibility == Visibility::Private}>{Visibility::Private}</option>
					</Select>
				</Control>
			</Field>
					<Field grouped=true>
						<Control><Button classes={"is-danger is-light"} onclick={discard}>{"Cancel"}</Button></Control>
						<Control><Button classes={"is-primary"} onclick={save_and_close}>{"Save"}</Button></Control>
					</Field>
				</ybc::Box>
			</div>
		</div>
	}
}
