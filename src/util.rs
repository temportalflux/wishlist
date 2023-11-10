pub mod error;
pub mod web_ext;

pub fn spawn_local<F, E>(target: &'static str, future: F)
where
	F: futures_util::Future<Output = Result<(), E>> + 'static,
	E: std::fmt::Debug + 'static,
{
	wasm_bindgen_futures::spawn_local(async move {
		if let Err(err) = future.await {
			log::error!(target: target, "{err:?}");
		}
	});
}
