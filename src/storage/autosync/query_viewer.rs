use github::{GithubClient, Query, RepositoryMetadata};

// Query github for the logged in user and all organizations they have access to.
pub struct QueryViewer {
	pub status: super::Status,
	pub client: GithubClient,
}
impl QueryViewer {
	pub async fn run(self) -> Result<(String, Option<RepositoryMetadata>), github::Error> {
		self.status.push_stage("Finding module owners", None);
		let search_params = github::SearchRepositoriesParams {
			query: Query::default()
				.keyed("user", "@me")
				.value(crate::storage::USER_DATA_REPO_NAME)
				.keyed("in", "name"),
			page_size: 1,
		};
		let (user, mut repositories) = self.client.search_repositories(search_params).await;

		self.status.pop_stage();
		Ok((user, repositories.pop()))
	}
}
