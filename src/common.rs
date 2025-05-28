use std::io::Cursor;

use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, ImageFormat};
use reqwest::Response;

// const TEN_MB: usize = 10 * 1024 * 1024;

/// Errors returned by the Moondream API
#[derive(Debug, thiserror::Error)]
pub enum ErrorResponse {
	#[error("Bad Request: Invalid parameters or image format")]
	BadRequest,

	#[error("Unauthorized: Invalid or missing API key")]
	Unauthorized,

	#[error("Payload Too Large: Image size exceeds limits")]
	PayloadTooLarge,

	#[error("Too Many Requests: Rate limit exceeded")]
	TooManyRequests,

	#[error("Internal Server Error")]
	InternalServerError,

	#[error("HTTP Error: {0}")]
	Http(#[from] reqwest::Error),

	#[error("Unexpected response: {0}")]
	Unexpected(String),

	#[error("JSON deserialization failed: {0}")]
	JsonError(#[from] serde_json::Error),
}

/// convert image to data URL base64 string (JPEG by default)
pub fn encode_image(image: &DynamicImage, fmt: ImageFormat) -> String {
	let mut buffer = Cursor::new(Vec::new());
	image.write_to(&mut buffer, fmt).unwrap();

	let mime = match fmt {
		ImageFormat::Png => "image/png",
		ImageFormat::Gif => "image/gif",
		_ => "image/jpeg",
	};

	let bytes = buffer.into_inner();

	// if bytes.len() > TEN_MB {
	// 	return Err("image is too large. max size is 10MB.".into());
	// }

	let encoded = general_purpose::STANDARD.encode(&bytes);
	format!("data:{};base64,{}", mime, encoded)
}

pub async fn handle_response(response: Response) -> Result<serde_json::Value, ErrorResponse> {
	match response.status().as_u16() {
		200 => Ok(response.json::<serde_json::Value>().await?),
		400 => Err(ErrorResponse::BadRequest),
		401 => Err(ErrorResponse::Unauthorized),
		413 => Err(ErrorResponse::PayloadTooLarge),
		429 => Err(ErrorResponse::TooManyRequests),
		500 => Err(ErrorResponse::InternalServerError),
		_ => Err(ErrorResponse::Unexpected(response.text().await.unwrap_or_default())),
	}
}

/// Describes the level of detail for generated captions.
#[derive(Debug, Clone, Copy)]
pub enum CaptionDetailLevel {
	/// A brief 1-2 sentence summary.
	/// Example: "A red car parked on a street."
	Short,

	/// A detailed description covering elements, context,
	/// colors, positioning, and other visual details.
	Normal,
}

impl CaptionDetailLevel {
	pub fn as_str(&self) -> &str {
		match self {
			CaptionDetailLevel::Short => "short",
			CaptionDetailLevel::Normal => "normal",
		}
	}
}

impl Default for CaptionDetailLevel {
	fn default() -> Self {
		CaptionDetailLevel::Normal
	}
}
