use super::query_rest;
use crate::{page::Route, session::Profile};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Deref, str::FromStr};
use yew::html::IntoPropValue;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GistId(String);
impl From<String> for GistId {
	fn from(value: String) -> Self {
		Self(value)
	}
}
impl Deref for GistId {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl GistId {
	pub fn get_url(&self) -> String {
		format!("https://api.github.com/gists/{}", self.0)
	}

	pub fn as_route(&self) -> Route {
		Route::List {
			gist_id: self.clone(),
		}
	}
}
impl FromStr for GistId {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self(s.to_owned()))
	}
}
impl std::fmt::Display for GistId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
	Public,
	Private,
}
impl Default for Visibility {
	fn default() -> Self {
		bool::default().into()
	}
}
impl From<bool> for Visibility {
	fn from(public: bool) -> Self {
		match public {
			true => Self::Public,
			false => Self::Private,
		}
	}
}
impl Into<bool> for Visibility {
	fn into(self) -> bool {
		match self {
			Self::Public => true,
			Self::Private => false,
		}
	}
}
impl Visibility {
	pub fn value(&self) -> &'static str {
		match self {
			Self::Public => "public",
			Self::Private => "private",
		}
	}

	pub fn as_tag(&self, classes: Option<yew::Classes>) -> yew::Html {
		use yew::html;
		let mut classes = classes.unwrap_or_default();
		match self {
			Self::Public => classes.push("is-info"),
			Self::Private => {
				classes.push("is-danger");
				classes.push("is-light");
			}
		}
		html! {<ybc::Tag classes={classes}>{self}</ybc::Tag>}
	}
}
impl FromStr for Visibility {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"public" => Ok(Self::Public),
			"private" => Ok(Self::Private),
			_ => Err(()),
		}
	}
}
impl std::fmt::Display for Visibility {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Public => "Anyone with link",
				Self::Private => "Only me",
			}
		)
	}
}
impl IntoPropValue<String> for Visibility {
	fn into_prop_value(self) -> String {
		self.value().to_owned()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GistInfo {
	pub id: GistId,
	pub title: String,
	pub owner_login: String,
	pub visibility: Visibility,
}

pub struct FetchProfile;
impl FetchProfile {
	pub async fn get() -> anyhow::Result<Profile> {
		let (app_user_data, lists) = Self::fetch_gists().await?;
		let app_user_data = match app_user_data {
			Some(data) => data,
			None => {
				let mut user_data = Gist::<AppUserData>::default();
				user_data.save().await?;
				user_data.id.unwrap()
			}
		};
		Ok(Profile {
			app_user_data,
			lists,
		})
	}

	async fn fetch_gists() -> anyhow::Result<(Option<GistId>, Vec<GistInfo>)> {
		#[derive(Deserialize, Debug)]
		struct Gist {
			id: String,
			description: String,
			owner: Owner,
			public: bool,
		}
		#[derive(Deserialize, Debug)]
		struct Owner {
			login: String,
		}
		static ENTRIES_PER_PAGE: usize = 10;
		let mut page = 1;
		let mut private_id = None;
		let mut lists = Vec::with_capacity(100);
		'fetch_gists: loop {
			let request = query_rest::<Vec<Gist>>(Method::GET, "/gists").with_query(&{
				let mut query = HashMap::new();
				query.insert("page", page);
				query.insert("per_page", ENTRIES_PER_PAGE);
				query
			});
			let gists = request.send().await?;
			log::debug!("Requested page {page}, found {} gists.", gists.len());
			page += 1;
			let is_last_page = gists.is_empty() || gists.len() < ENTRIES_PER_PAGE;
			for gist in gists.into_iter() {
				if private_id.is_none() && gist.description.starts_with(AppUserData::prefix()) {
					private_id = Some(GistId::from(gist.id));
				} else if gist.description.starts_with(List::prefix()) {
					let name = List::parse_name_from(&gist.description).to_owned();
					lists.push(GistInfo {
						id: GistId::from(gist.id),
						title: name,
						owner_login: gist.owner.login,
						visibility: gist.public.into(),
					});
				}
			}
			if is_last_page {
				break 'fetch_gists;
			}
		}
		lists.sort_by(|a, b| a.title.partial_cmp(&b.title).unwrap());
		Ok((private_id, lists))
	}
}

pub trait GistDocument {
	/// The prefix used for this document-type in the description.
	fn prefix() -> &'static str;

	fn name_prefix() -> String {
		format!("{} - ", Self::prefix())
	}

	/// Return the name of the document, for use in the description.
	fn name(&self) -> &str
	where
		Self: Sized;

	/// Return the document description, as a combination of prefix and name.
	fn description(&self) -> String
	where
		Self: Sized,
	{
		format!("{}{}", Self::name_prefix(), self.name())
	}

	fn parse_name_from<'s>(description: &'s str) -> &'s str {
		description.strip_prefix(&Self::name_prefix()).unwrap()
	}

	/// Serialize as a kdl document
	fn as_document(&self) -> kdl::KdlDocument;

	/// Deserialize from a kdl document
	fn from_kdl(doc: &kdl::KdlDocument) -> anyhow::Result<Self>
	where
		Self: Sized;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Gist<T> {
	pub id: Option<GistId>,
	pub description: String,
	pub visibility: Visibility,
	pub owner_login: String,
	pub file: T,
}
impl<T> From<T> for Gist<T>
where
	T: GistDocument,
{
	fn from(document: T) -> Self {
		Self {
			id: None,
			description: document.description(),
			visibility: Visibility::Private,
			owner_login: String::new(),
			file: document,
		}
	}
}
impl<T> Gist<T>
where
	T: GistDocument,
{
	pub fn with_visibility(mut self, value: Visibility) -> Self {
		self.visibility = value;
		self
	}

	pub fn with_owner(mut self, value: String) -> Self {
		self.owner_login = value;
		self
	}

	pub async fn fetch(id: GistId) -> anyhow::Result<Self> {
		#[allow(dead_code)]
		#[derive(Deserialize, Debug)]
		struct Root {
			id: String,
			description: String,
			owner: Owner,
			public: bool,
			updated_at: String,
			files: HashMap<String, File>,
		}
		#[derive(Deserialize, Debug)]
		struct Owner {
			login: String,
		}
		#[derive(Deserialize, Debug)]
		struct File {
			content: String,
		}
		let request = query_rest::<Root>(Method::GET, &format!("/gists/{id}"));
		log::debug!("Request: {request:?}");
		let mut data = request.send().await?;
		let raw_file = data.files.remove(&data.description).unwrap();
		let kdl: kdl::KdlDocument = raw_file.content.parse()?;
		let document = T::from_kdl(&kdl)?;
		Ok(Self {
			id: Some(data.id.into()),
			description: data.description,
			visibility: data.public.into(),
			owner_login: data.owner.login,
			file: document,
		})
	}

	pub async fn save(&mut self) -> anyhow::Result<()> {
		#[derive(Debug, Serialize)]
		struct Body {
			description: String,
			public: bool,
			files: HashMap<String, File>,
		}
		#[derive(Debug, Serialize)]
		struct File {
			content: String,
		}
		#[derive(Debug, Deserialize)]
		struct GistData {
			id: String,
		}
		let body = Body {
			description: self.description.clone(),
			public: self.visibility.into(),
			files: {
				let mut files = HashMap::new();
				files.insert(
					self.description.clone(),
					File {
						content: self.file.as_document().to_string(),
					},
				);
				files
			},
		};

		let endpoint = match &self.id {
			Some(id) => format!("/gists/{}", id.to_string()),
			None => "/gists".to_owned(),
		};
		let request = query_rest::<GistData>(Method::POST, &endpoint).with_json(&body);
		log::debug!("Request: {request:?} Body: {body:?}");
		let gist_data = request.send().await?;
		self.id = Some(gist_data.id.into());
		Ok(())
	}
}

/// Metadata about a user that is stored in the "wishlist::private" file.
/// Contains information like the items reserved from another user's wishlist.
#[derive(Default)]
pub struct AppUserData {}
impl GistDocument for AppUserData {
	fn prefix() -> &'static str {
		"wishlist::private"
	}

	fn name(&self) -> &str {
		"App User Data"
	}

	fn as_document(&self) -> kdl::KdlDocument {
		let mut doc = kdl::KdlDocument::new();
		doc.nodes_mut().push({
			let mut node = kdl::KdlNode::new("name");
			node.entries_mut()
				.push(kdl::KdlEntry::new("name goes here"));
			node
		});
		doc
	}

	fn from_kdl(_doc: &kdl::KdlDocument) -> anyhow::Result<Self> {
		Ok(Self {})
	}
}

/// A wishlist stored on github with the "wishlist::document" prefix.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct List {
	pub name: String,
}
impl List {
	pub fn new(name: impl Into<String>) -> Self {
		Self { name: name.into() }
	}
}
impl GistDocument for List {
	fn prefix() -> &'static str {
		"wishlist::document"
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn as_document(&self) -> kdl::KdlDocument {
		let mut doc = kdl::KdlDocument::new();
		doc.nodes_mut().push({
			let mut node = kdl::KdlNode::new("name");
			node.entries_mut()
				.push(kdl::KdlEntry::new(self.name.clone()));
			node
		});
		doc
	}

	fn from_kdl(doc: &kdl::KdlDocument) -> anyhow::Result<Self> {
		let name = doc.get("name").expect("missing name node");
		let name = name.get(0).expect("missing value for name node");
		let name = name
			.value()
			.as_string()
			.expect("value for name node is not a string")
			.to_owned();
		Ok(Self { name })
	}
}
