use crate::session::Profile;

use super::query_rest;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistId(String);
impl From<String> for GistId {
	fn from(value: String) -> Self {
		Self(value)
	}
}
impl ToString for GistId {
	fn to_string(&self) -> String {
		self.0.clone()
	}
}
impl GistId {
	pub fn get_url(&self) -> String {
		format!("https://api.github.com/gists/{}", self.0)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistInfo {
	pub id: GistId,
	pub title: String,
	pub owner_login: String,
}

pub struct FetchProfile;
impl FetchProfile {
	pub async fn get() -> anyhow::Result<Profile> {
		let (app_user_data, lists) = Self::fetch_gists().await?;
		let app_user_data = match app_user_data {
			Some(data) => data,
			None => {
				let mut user_data = AppUserData::new_gist();
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
					let name = gist
						.description
						.strip_prefix(&format!("{} - ", List::prefix()))
						.unwrap()
						.to_owned();
					lists.push(GistInfo {
						id: GistId::from(gist.id),
						title: name,
						owner_login: gist.owner.login,
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
	fn prefix() -> &'static str;
	fn as_document(&self) -> kdl::KdlDocument;
}

pub struct Gist<T> {
	pub id: Option<GistId>,
	pub description: String,
	pub public: bool,
	pub file: T,
}
impl<T> Gist<T>
where
	T: GistDocument,
{
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
			public: self.public,
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
impl AppUserData {
	pub fn new_gist() -> Gist<Self> {
		Gist {
			id: None,
			description: format!("{} - App User Data", Self::prefix()),
			public: false,
			file: Self {},
		}
	}
}
impl GistDocument for AppUserData {
	fn prefix() -> &'static str {
		"wishlist::private"
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
}

/// A wishlist stored on github with the "wishlist::public" prefix.
pub struct List {
	name: String,
	public: bool,
}
impl List {
	pub fn new(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			public: false,
		}
	}
}
impl List {
	pub fn into_gist(self) -> Gist<List> {
		Gist {
			id: None,
			description: format!("{} - {}", Self::prefix(), self.name),
			public: self.public,
			file: self,
		}
	}
}
impl GistDocument for List {
	fn prefix() -> &'static str {
		"wishlist::public"
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
}
