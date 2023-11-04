use crate::{data, database::{Database, User, List, ListId}};
use derivative::Derivative;
use github::ChangedFileStatus;
use kdlize::AsKdl;
use std::{cell::RefCell, rc::Rc};
use yew::{html::ChildrenProps, prelude::*};
use yew_hooks::*;

mod query_viewer;
use query_viewer::*;
mod generate_repo;
use generate_repo::*;

#[derive(Clone)]
pub struct Channel(Rc<RequestChannel>);
impl PartialEq for Channel {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}
impl std::ops::Deref for Channel {
	type Target = RequestChannel;

	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}

pub struct RequestChannel {
	send_req: async_channel::Sender<Request>,
	recv_req: async_channel::Receiver<Request>,
}
impl RequestChannel {
	pub fn try_send_req(&self, req: Request) {
		let _ = self.send_req.try_send(req);
	}
}

#[derive(Debug)]
pub enum Request {
	UpdateLists,
}

#[derive(Clone, Derivative)]
#[derivative(PartialEq)]
pub struct Status {
	#[derivative(PartialEq = "ignore")]
	rw_internal: Rc<RefCell<StatusState>>,
	r_external: UseStateHandle<StatusState>,
}
impl std::fmt::Debug for Status {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Status")
			.field("State", &self.rw_internal)
			.field("Display", &self.r_external)
			.finish()
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
struct StatusState {
	stages: Vec<Stage>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Stage {
	pub title: AttrValue,
	pub progress: Option<Progress>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Progress {
	pub max: usize,
	pub progress: usize,
}

impl Status {
	fn mutate(&self, perform: impl FnOnce(&mut StatusState)) {
		let mut state = self.rw_internal.borrow_mut();
		perform(&mut *state);
		self.r_external.set(state.clone());
	}

	pub fn push_stage(&self, title: impl Into<AttrValue>, max_progress: Option<usize>) {
		self.mutate(move |state| {
			state.stages.push(Stage {
				title: title.into(),
				progress: max_progress.map(|max| Progress { max, progress: 0 }),
			});
		});
	}

	pub fn pop_stage(&self) {
		self.mutate(move |state| {
			state.stages.pop();
		});
	}

	pub fn set_progress_max(&self, max: usize) {
		self.mutate(move |state| {
			let Some(stage) = state.stages.last_mut() else {
				log::error!(target: "autosync", "status has no stages");
				return;
			};
			let Some(progress) = &mut stage.progress else {
				log::error!(target: "autosync", "{stage:?} has no progress");
				return;
			};
			progress.max = max;
		});
	}

	pub fn increment_progress(&self) {
		self.mutate(move |state| {
			let Some(stage) = state.stages.last_mut() else {
				log::error!(target: "autosync", "status has no stages");
				return;
			};
			let Some(progress) = &mut stage.progress else {
				log::error!(target: "autosync", "{stage:?} has no progress");
				return;
			};
			progress.progress = progress.max.min(progress.progress + 1);
		});
	}

	pub fn is_active(&self) -> bool {
		!self.r_external.stages.is_empty()
	}

	pub fn stages(&self) -> &Vec<Stage> {
		&self.r_external.stages
	}
}

#[derive(thiserror::Error, Debug, Clone)]
enum StorageSyncError {
	#[error(transparent)]
	Database(#[from] database::Error),
	#[error(transparent)]
	StorageError(#[from] github::Error),
}

#[function_component]
pub fn Provider(props: &ChildrenProps) -> Html {
	let database = use_context::<Database>().unwrap();
	let channel = Channel(use_memo((), |_| {
		let (send_req, recv_req) = async_channel::unbounded();
		RequestChannel { send_req, recv_req }
	}));
	let status = Status {
		rw_internal: Rc::new(RefCell::new(StatusState::default())),
		r_external: use_state_eq(|| StatusState { ..Default::default() }),
	};
	use_async_with_options(
		{
			let database = database.clone();
			let recv_req = channel.recv_req.clone();
			let status = status.clone();
			async move {
				while let Ok(req) = recv_req.recv().await {
					if let Err(err) = process_request(req, &database, &status).await {
						log::error!(target: "autosync", "{err:?}");
					}
				}
				Ok(()) as Result<(), StorageSyncError>
			}
		},
		UseAsyncOptions::enable_auto(),
	);

	html! {
		<ContextProvider<Channel> context={channel}>
			<ContextProvider<Status> context={status}>
				{props.children.clone()}
			</ContextProvider<Status>>
		</ContextProvider<Channel>>
	}
}

async fn process_request(req: Request, database: &Database, status: &Status) -> Result<(), StorageSyncError> {
	let auth_status = yewdux::dispatch::get::<crate::auth::Status>();
	let Some(storage) = crate::storage::get(&*auth_status) else {
		log::error!(target: "autosync", "No storage available, cannot progess request {req:?}");
		return Ok(());
	};

	match req {
		Request::UpdateLists => {}
	}

	// NOTE: The initial implementation used gists. We arent going to do that anymore b/c other users cannot collaborate on it which means no reserving of items.

	status.push_stage("Checking authentiation", None);
	let query_viewer = QueryViewer {
		status: status.clone(),
		client: storage.clone(),
	};
	let (user, data_repo) = query_viewer.run().await?;
	status.pop_stage();

	let repository = match data_repo {
		Some(repo) => repo,
		None => {
			status.push_stage("Generating user data storage", None);

			// Generate the repository
			let generate_repo = GenerateDataRepo {
				status: status.clone(),
				client: storage.clone(),
			};
			let response = generate_repo.run().await?;

			// Find the tree-id of the repository for scanning, because it isnt provided in the create repo response.
			let search_params = github::SearchRepositoriesParams {
				query: github::Query::default()
					.keyed("user", &response.owner)
					.value(&response.name)
					.keyed("in", "name"),
				page_size: 1,
			};
			let (_, repositories) = storage.search_repositories(search_params).await;
			let tree_id = match repositories.into_iter().next() {
				Some(metadata) => metadata.tree_id,
				None => {
					let resp_err = github::Error::InvalidResponse(format!("Empty repository metadata").into());
					return Err(resp_err.into());
				}
			};

			status.pop_stage();

			github::RepositoryMetadata {
				owner: response.owner,
				name: response.name,
				is_private: false,
				version: response.remote_version,
				tree_id,
				root_trees: Vec::default(),
			}
		}
	};

	enum FileKind {
		User,
		List,
	}
	
	// Check for a user in the database and if there is a version mismatch.
	// No local user means we must scan repository and install (download current state).
	// Version mismatch means we need to ask storage for what the updates are.
	let mut users = Vec::<User>::default();
	let mut lists = Vec::<List>::default();
	let mut removed_lists = Vec::<ListId>::default();
	match database.get::<User>(user.as_str()).await? {
		// need to install
		None => {
			let mut files = Vec::<(FileKind, String, String)>::default();
		
			// We only scan the top level because the app only saves wishlists to the top-most root.
			// There are no subdirectories generated by the app to recurse through.
			status.push_stage("Scanning Storage", None);
			let args = github::repos::tree::Args {
				owner: repository.owner.as_str(),
				repo: repository.name.as_str(),
				tree_id: repository.tree_id.as_str(),
			};
			for entry in storage.get_tree(args).await? {
				if entry.is_tree || !entry.path.ends_with(".kdl") {
					continue;
				}
				
				// We only have 2 types of files: user data and lists. Lets just hard-code some typing here.
				let kind = match entry.path.ends_with("user.kdl") {
					true => FileKind::User,
					false => FileKind::List,
				};
				files.push((kind, entry.id, entry.path));
			}
			status.pop_stage();

			status.push_stage("Downloading Storage", Some(files.len()));
			for (file_kind, file_id, remote_path) in files {
				let path = std::path::Path::new(remote_path.as_str());
				let args = github::repos::contents::get::Args {
					owner: &repository.owner,
					repo: &repository.name,
					path,
					version: &repository.version,
				};
				let content = storage.get_file_content(args).await?;

				match file_kind {
					FileKind::User => {
						users.push(User {
							login: user.clone(),
							file_id: Some(file_id),
							kdl: content,
							root_tree_id: repository.tree_id.clone(),
							local_version: repository.version.clone(),
						});
					}
					FileKind::List => {
						lists.push(List {
							id: ListId::from_path(&user, path),
							file_id: Some(file_id),
							kdl: content,
							local_version: repository.version.clone(),
						});
					}
				}

				status.increment_progress();
			}
			status.pop_stage();
		}
		// need to ask for diffs
		Some(mut user) if user.local_version != repository.version => {
			status.push_stage("Downloading Updates", None);
			
			let args = github::repos::compare::Args {
				owner: repository.owner.as_str(),
				repo: repository.name.as_str(),
				commit_start: user.local_version.as_str(),
				commit_end: repository.version.as_str(),
			};
			let changed_file_paths = storage.get_files_changed(args).await?;
			status.set_progress_max(changed_file_paths.len());
			for changed_file in changed_file_paths {
				let path = std::path::Path::new(changed_file.path.as_str());
				match changed_file.status {
					ChangedFileStatus::Added
					| ChangedFileStatus::Modified
					| ChangedFileStatus::Renamed
					| ChangedFileStatus::Copied
					| ChangedFileStatus::Changed => {
						let args = github::repos::contents::get::Args {
							owner: repository.owner.as_str(),
							repo: repository.name.as_str(),
							path,
							version: repository.version.as_str(),
						};
						let content = storage.get_file_content(args).await?;
						let file_kind = match changed_file.path.ends_with("user.kdl") {
							true => FileKind::User,
							false => FileKind::List,
						};
						match file_kind {
							FileKind::User => {
								user.kdl = content;
							}
							FileKind::List => {
								lists.push(List {
									id: ListId::from_path(&user.login, path),
									file_id: Some(changed_file.file_id),
									kdl: content,
									local_version: repository.version.clone(),
								});
							}
						}
					}
					ChangedFileStatus::Removed => {
						if !changed_file.path.ends_with("user.kdl") {
							removed_lists.push(ListId::from_path(&user.login, path));
						}
					}
					ChangedFileStatus::Unchanged => {}
				}

				status.increment_progress();
			}

			user.local_version = repository.version.clone();
			users.push(user);
			
			status.pop_stage();
		}
		// no update required for this user's data
		_ => {}
	}

	// TODO: Will need to look at the user data to find what lists the user is currently invited to.
	// Check the version of those lists and download updates. Also filter out any lists that have uninvited this user
	// (i.e. this user is no longer on that lists invites).

	if !users.is_empty() || !lists.is_empty() || !removed_lists.is_empty() {
		status.push_stage("Installing Downloaded Files", None);
		database
			.mutate(move |transaction| {
				use database::{ObjectStoreExt, TransactionExt};
				Box::pin(async move {
					
					let user_store = transaction.object_store_of::<User>()?;
					for user in users {
						user_store.put_record(&user).await?;
					}

					let list_store = transaction.object_store_of::<List>()?;
					for list in lists {
						list_store.put_record(&list).await?;
					}
					for list_id in removed_lists {
						list_store.delete_record(list_id.to_string()).await?;
					}
					
					Ok(())
				})
			})
			.await?;
		status.pop_stage();
	}

	Ok(())
}
