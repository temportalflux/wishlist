use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Serialize};

pub struct Response<T> {
	builder: RequestBuilder,
	marker: std::marker::PhantomData<T>,
}
impl<T> std::fmt::Debug for Response<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.builder.fmt(f)
	}
}
impl<T> Response<T>
where
	T: DeserializeOwned,
{
	pub fn from(builder: RequestBuilder) -> Self {
		Self {
			builder,
			marker: Default::default(),
		}
	}

	pub fn with_query<Q>(mut self, query: &Q) -> Self
	where
		Q: Serialize + ?Sized,
	{
		self.builder = self.builder.query(query);
		self
	}

	pub fn with_json<Q>(mut self, json: &Q) -> Self
	where
		Q: Serialize + ?Sized,
	{
		self.builder = self.builder.json(json);
		self
	}

	pub async fn send(self) -> anyhow::Result<T> {
		let response: reqwest::Response = self.builder.send().await?;
		let text = response.text().await?;
		let output = match serde_json::from_str(&text) {
			Ok(data) => data,
			Err(err) => {
				return Err(InvalidJson(text, err))?;
			}
		};
		Ok(output)
	}
}

#[derive(thiserror::Error, Debug)]
pub struct InvalidJson(pub String, pub serde_json::Error);
impl std::fmt::Display for InvalidJson {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Invalid json: {:?}\nError: {:?}", self.0, self.1)
	}
}
