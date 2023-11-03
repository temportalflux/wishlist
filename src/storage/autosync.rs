use crate::{data, database::Database};
use derivative::Derivative;
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

	match data_repo {
		None => {
			status.push_stage("Generating user data storage", None);

			let generate_repo = GenerateDataRepo {
				status: status.clone(),
				client: storage.clone(),
			};
			let response = generate_repo.run().await?;

			// user data was created in remote, create a corresponding entry in database
			let user = crate::database::User {
				login: user.clone(),
				file_id: Some(response.user_file_id),
				kdl: response.user_content,
				local_version: response.remote_version.clone(),
				remote_version: response.remote_version,
			};
			database
				.mutate(move |transaction| {
					use database::{ObjectStoreExt, TransactionExt};
					Box::pin(async move {
						let user_store = transaction.object_store_of::<crate::database::User>()?;
						user_store.put_record(&user).await?;
						Ok(())
					})
				})
				.await?;

			status.pop_stage();
		}
		Some(repo) => {

			let local_user = database.get::<crate::database::User>(user.as_str()).await?;
			match local_user {
				Some(mut user) => {
					// TODO: update remote version of user in database
					user.remote_version = repo.version.clone();
					//user
				}
				None => {
					// TODO: grab user.kdl from repo so we can get the file_id & kdl content
					let args = github::repos::contents::get::Args {
						owner: &repo.owner,
						repo: &repo.name,
						path: std::path::Path::new("user.kdl"),
						version: &repo.version,
					};
					let content = storage.get_file_content(args).await?;

				}
			};

			// TODO: if remote version != local version, fetch and download changes to lists
			// TODO: fetch remote versions of external lists, and download any changes
		}
	}

	Ok(())
}
