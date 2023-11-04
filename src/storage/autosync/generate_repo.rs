use crate::storage::{DATA_REPO_TOPIC, USER_DATA_REPO_NAME};
use github::{repos, GithubClient};
use kdlize::{ext::NodeExt, AsKdl};

// Create the homebrew repo on the github client viewer (the user that is logged in).
// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user
// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#create-or-update-file-contents
pub struct GenerateDataRepo {
	pub status: super::Status,
	pub client: GithubClient,
}
pub struct GenerateDataRepoResponse {
	pub owner: String,
	pub name: String,
	pub user_file_id: String,
	pub user_content: String,
	pub remote_version: String,
}
impl GenerateDataRepo {
	pub async fn run(self) -> Result<GenerateDataRepoResponse, github::Error> {
		let create_repo = repos::create::Args {
			org: None,
			name: USER_DATA_REPO_NAME,
			private: false,
		};
		let owner = self.client.create_repo(create_repo).await?;

		let set_topics = repos::set_topics::Args {
			owner: owner.as_str(),
			repo: USER_DATA_REPO_NAME,
			topics: vec![DATA_REPO_TOPIC.to_owned()],
		};
		self.client.set_repo_topics(set_topics).await?;

		let content = crate::data::User::default()
			.as_kdl()
			.build("user")
			.to_doc_string_unescaped();
		let args = github::repos::contents::update::Args {
			repo_org: &owner,
			repo_name: USER_DATA_REPO_NAME,
			path_in_repo: std::path::Path::new("user.kdl"),
			commit_message: "Initialize user data",
			content: &content,
			file_id: None,
			branch: None,
		};
		let response = self.client.create_or_update_file(args).await?;

		Ok(GenerateDataRepoResponse {
			owner,
			name: USER_DATA_REPO_NAME.into(),
			user_file_id: response.file_id,
			user_content: content,
			remote_version: response.version,
		})
	}
}
