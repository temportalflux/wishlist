use super::query_rest;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::Future, pin::Pin};

#[derive(Debug)]
pub struct GistId(String);
impl From<String> for GistId {
	fn from(value: String) -> Self {
		Self(value)
	}
}
impl GistId {
	pub fn get_url(&self) -> String {
		format!("https://api.github.com/gists/{}", self.0)
	}
}

pub async fn find_gist() -> anyhow::Result<(Option<GistId>, Vec<GistId>)> {
	#[derive(Deserialize, Debug)]
	struct Gist {
		id: String,
		url: String,
		description: String,
		files: HashMap<String, File>,
		public: bool,
	}
	#[derive(Deserialize, Debug)]
	struct File {
		filename: String,
		raw_url: String,
	}
	static ENTRIES_PER_PAGE: usize = 10;
	let mut page = 1;
	let mut private_id = None;
	let mut list_ids = Vec::with_capacity(100);
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
				list_ids.push(GistId::from(gist.id));
			}
		}
		if is_last_page {
			break 'fetch_gists;
		}
	}
	Ok((private_id, list_ids))
}

pub trait GistDocument {
	fn prefix() -> &'static str;
	fn as_document(&self) -> kdl::KdlDocument;
}

pub struct Gist<T> {
	pub id: Option<String>,
	pub description: String,
	pub public: bool,
	pub files: HashMap<String, T>,
}
impl<T> Gist<T>
where
	T: GistDocument,
{
	pub fn get_mut(&mut self, file_name: &str) -> Option<&mut T> {
		self.files.get_mut(file_name)
	}

	pub async fn save(&mut self) -> anyhow::Result<()> {
		#[derive(Serialize)]
		struct Body {
			description: String,
			public: bool,
			files: HashMap<String, File>,
		}
		#[derive(Serialize)]
		struct File {
			content: String,
		}
		#[derive(Deserialize)]
		struct GistData {
			id: String,
		}
		let body = Body {
			description: self.description.clone(),
			public: self.public,
			files: self
				.files
				.iter()
				.map(|(name, item)| {
					(
						name.clone(),
						File {
							content: item.as_document().to_string(),
						},
					)
				})
				.collect(),
		};

		let endpoint = match &self.id {
			Some(id) => format!("/gists/{id}"),
			None => "/gists".to_owned(),
		};
		let request = query_rest::<GistData>(Method::POST, &endpoint).with_json(&body);
		let gist_data = request.send().await?;
		self.id = Some(gist_data.id);
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
			files: {
				let mut files = HashMap::new();
				files.insert(Self::prefix().to_owned(), Self {});
				files
			},
		}
	}
}
impl GistDocument for AppUserData {
	fn prefix() -> &'static str {
		"wishlist::private"
	}

	fn as_document(&self) -> kdl::KdlDocument {
		let mut doc = kdl::KdlDocument::new();
		/*
		doc.nodes_mut().push({
			let mut node = kdl::KdlNode::new("name");
			node.entries_mut().push(kdl::KdlEntry::new(self.name.clone()));
			node
		});
		*/
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
		Self { name: name.into(), public: false }
	}
}
impl List {
	pub fn into_gist(self) -> Gist<List> {
		let public = self.public;
		let description = format!("{} - {}", Self::prefix(), self.name);
		let mut files = HashMap::new();
		files.insert(description.clone(), self);
		Gist {
			id: None,
			description,
			public,
			files,
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
