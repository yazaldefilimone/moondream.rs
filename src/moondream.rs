use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use image::{DynamicImage, ImageFormat};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::io::Cursor;

#[derive(Default)]
pub enum CaptionDetailLevel {
	#[default]
	Short,
	Medium,
	Long,
}

impl CaptionDetailLevel {
	pub fn as_str(&self) -> &'static str {
		match self {
			CaptionDetailLevel::Short => "low",
			CaptionDetailLevel::Medium => "medium",
			CaptionDetailLevel::Long => "high",
		}
	}
}

#[derive(Deserialize)]
pub struct CaptionResponse {
	pub caption: String,
}

#[derive(Deserialize)]
pub struct QueryResponse {
	pub answer: String,
}

#[derive(Deserialize)]
pub struct DetectResponse {
	pub objects: serde_json::Value,
}

#[derive(Deserialize)]
pub struct PointResponse {
	pub points: serde_json::Value,
}

pub struct Moondream {
	api_key: String,
	client: Client,
}

impl Moondream {
	pub fn new(api_key: impl Into<String>) -> Self {
		Self { api_key: api_key.into(), client: Client::new() }
	}

	fn encode_image(&self, image: &DynamicImage) -> Result<String> {
		let mut buffer = Cursor::new(Vec::new());
		image.write_to(&mut buffer, ImageFormat::Jpeg)?;
		let encoded = base64::encode(buffer.get_ref());
		Ok(format!("data:image/jpeg;base64,{}", encoded))
	}

	async fn post_json<T: for<'de> Deserialize<'de>>(
		&self,
		url: &str,
		body: serde_json::Value,
	) -> Result<T> {
		let response = self
			.client
			.post(url)
			.header("Authorization", format!("Bearer {}", self.api_key))
			.json(&body)
			.send()
			.await?
			.error_for_status()?;

		Ok(response.json::<T>().await?)
	}

	pub async fn point(&self, image: &DynamicImage, object: &str) -> Result<PointResponse> {
		let url = "https://api.moondream.ai/v1/point";
		let image_url = self.encode_image(image)?;
		let body = json!({ "image_url": image_url, "object": object });
		self.post_json(url, body).await
	}

	pub async fn detect(&self, image: &DynamicImage, object: &str) -> Result<DetectResponse> {
		let url = "https://api.moondream.ai/v1/detect";
		let image_url = self.encode_image(image)?;
		let body = json!({ "image_url": image_url, "object": object });
		self.post_json(url, body).await
	}

	pub async fn query(&self, image: &DynamicImage, question: &str) -> Result<QueryResponse> {
		let url = "https://api.moondream.ai/v1/query";
		let image_url = self.encode_image(image)?;
		let body = json!({ "image_url": image_url, "question": question, "stream": false });
		self.post_json(url, body).await
	}

	pub async fn query_stream<F>(
		&self,
		image: &DynamicImage,
		question: &str,
		mut on_chunk: F,
	) -> Result<()>
	where
		F: FnMut(&str) + Send,
	{
		let url = "https://api.moondream.ai/v1/query";
		let image_url = self.encode_image(image)?;
		let body = json!({ "image_url": image_url, "question": question, "stream": true });

		let response = self
			.client
			.post(url)
			.header("Authorization", format!("Bearer {}", self.api_key))
			.json(&body)
			.send()
			.await?
			.error_for_status()?;

		let mut stream = response.bytes_stream();
		let mut buffer = String::new();

		while let Some(chunk) = stream.next().await {
			let bytes = chunk?;
			buffer.push_str(std::str::from_utf8(&bytes)?);
			while let Some(pos) = buffer.find('\n') {
				let line = buffer[..pos].trim();
				buffer = buffer[(pos + 1)..].to_string();
				if let Some(data) = line.strip_prefix("data: ") {
					if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
						if let Some(chunk) = json.get("chunk").and_then(|c| c.as_str()) {
							on_chunk(chunk);
						}
					}
				}
			}
		}

		Ok(())
	}

	pub async fn caption(
		&self,
		image: &DynamicImage,
		detail_level: Option<CaptionDetailLevel>,
	) -> Result<CaptionResponse> {
		let url = "https://api.moondream.ai/v1/caption";
		let image_url = self.encode_image(image)?;
		let length = detail_level.unwrap_or_default().as_str();
		let body = json!({ "image_url": image_url, "length": length, "stream": false });
		self.post_json(url, body).await
	}

	pub async fn caption_stream<F>(
		&self,
		image: &DynamicImage,
		detail_level: Option<CaptionDetailLevel>,
		mut on_chunk: F,
	) -> Result<()>
	where
		F: FnMut(&str) + Send,
	{
		let url = "https://api.moondream.ai/v1/caption";
		let image_url = self.encode_image(image)?;
		let length = detail_level.unwrap_or_default().as_str();
		let body = json!({ "image_url": image_url, "length": length, "stream": true });

		let response = self
			.client
			.post(url)
			.header("Authorization", format!("Bearer {}", self.api_key))
			.json(&body)
			.send()
			.await?
			.error_for_status()?;

		let mut stream = response.bytes_stream();
		let mut buffer = String::new();

		while let Some(chunk) = stream.next().await {
			let bytes = chunk?;
			buffer.push_str(std::str::from_utf8(&bytes)?);
			while let Some(pos) = buffer.find('\n') {
				let line = buffer[..pos].trim();
				buffer = buffer[(pos + 1)..].to_string();
				if let Some(data) = line.strip_prefix("data: ") {
					if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
						if let Some(chunk) = json.get("chunk").and_then(|c| c.as_str()) {
							on_chunk(chunk);
						}
					}
				}
			}
		}

		Ok(())
	}
}
