use std::collections::HashMap;

use super::query_rest;
use reqwest::Method;
use serde::Deserialize;

pub async fn find_gist() -> anyhow::Result<Option<String>> {
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
	let gists = query_rest::<Vec<Gist>>(Method::GET, "/gists", Some(|req| {
		req.query(&{
			let mut query = HashMap::new();
			query.insert("page", 1);
			query.insert("per_page", 100);
			query
		})
	})).send().await?;
	log::debug!("{:?}", gists);
	for gist in gists.into_iter() {
		if gist.description.starts_with("wishlist-application") {
			return Ok(Some(gist.id));
		}
	}
	Ok(None)
}
