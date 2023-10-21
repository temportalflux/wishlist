use derivative::Derivative;
use std::{cell::RefCell, rc::Rc};
use yew::{html::ChildrenProps, prelude::*};
use yew_hooks::*;
use crate::database::Database;

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

	status.push_stage("Checking authentiation", None);

	let query_viewer = QueryViewer {
		status: status.clone(),
		client: storage.clone(),
	};
	let (user, data_repo) = query_viewer.run().await?;

	if data_repo.is_none() {
		let generate_repo = GenerateDataRepo {
			status: status.clone(),
			client: storage.clone(),
		};
		generate_repo.run().await?;
	}
	
	// TODO: Update the data_repo if its out of date
	// TODO: From the database (updated from prev stage), read all wishlist ids that the user has access to (those they own and those they were invited to).
	// update all this data by first checking the remote version, and then getting file content/parsing kdl.
	// TODO: When an out of date list is detected, its remote version is updated. We wont hold a local cache for data repos.
	// TODO: Also need to update the user with their data from the datarepo

	// NOTE: The initial implementation used gists. We arent going to do that anymore b/c other users cannot collaborate on it which means no reserving of items.

	status.pop_stage();

	Ok(())
}
