use std::rc::Rc;
use yew::prelude::*;
use yew_hooks::{use_mount, UseAsyncState};

pub struct AsyncHandle<T, E> {
	state: UseStateHandle<UseAsyncState<T, E>>,
	run: Rc<dyn Fn()>,
}
impl<T, E> AsyncHandle<T, E> {
	pub fn run(&self) {
		(*self.run)();
	}
}
impl<T, E> std::ops::Deref for AsyncHandle<T, E> {
	type Target = UseAsyncState<T, E>;

	fn deref(&self) -> &Self::Target {
		&self.state
	}
}

#[hook]
pub fn use_async<F, T, E>(run_first_mount: bool, make_future: F) -> AsyncHandle<T, E>
where
	F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>> + 'static,
	T: 'static,
	E: 'static,
{
	let state = use_state(|| UseAsyncState {
		loading: false,
		data: None,
		error: None,
	});
	let make_future = Rc::new(make_future);
	let run = {
		let state = state.clone();
		Rc::new(move || {
			state.set(UseAsyncState {
				loading: true,
				data: None,
				error: None,
			});
			let async_state = state.clone();
			let make_future = make_future.clone();
			wasm_bindgen_futures::spawn_local(async move {
				let final_state = match make_future().await {
					Ok(data) => UseAsyncState {
						loading: false,
						data: Some(data),
						error: None,
					},
					Err(err) => UseAsyncState {
						loading: false,
						data: None,
						error: Some(err),
					},
				};
				async_state.set(final_state);
			})
		})
	};
	let run_on_mount = run.clone();
	use_mount(move || {
		if run_first_mount {
			run_on_mount();
		}
	});
	AsyncHandle { state, run }
}
