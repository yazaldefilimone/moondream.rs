use futures_util::StreamExt;
use image::open;
use moondream::Moondream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// replace this with your actual API key
	let api_key = "ey";

	// load an image from disk
	let image = open("assets/demo.jpg")?;

	// Create the Moondream client
	let moondream = Moondream::new(api_key);

	// example 1: object detection
	// Detect "person" in the image
	let objects = moondream.detect(&image, "person").await?;
	println!("Detected objects: {:#?}", objects);

	// example 2: Object Pointing
	// Get center points for "face" in the image
	let points = moondream.point(&image, "face").await?;
	println!("Detected points: {:#?}", points);

	// example 3: query (VQA)
	// Ask a question about the image
	let answer = moondream.query(&image, "What is the girl doing?").await?;
	println!("Answer: {}", answer);

	// example 4: query stream
	// stream an answer about the image
	println!("Streaming answer:");
	let mut stream = moondream.query_stream(&image, "What is this?").await?;

	while let Some(chunk) = stream.next().await {
		let bytes = chunk?;
		let text = std::str::from_utf8(&bytes)?;
		println!("chunk: {}", text);
	}
	println!("");

	// example 5: caption
	// Generate a caption for the image
	let caption = moondream.caption(&image, None).await?;
	println!("Caption: {}", caption);

	// example 6: caption stream
	// stream a caption for the image
	println!("Streaming caption:");
	let mut stream = moondream.caption_stream(&image, None).await?;

	while let Some(chunk) = stream.next().await {
		let bytes = chunk?;
		let text = std::str::from_utf8(&bytes)?;
		println!("chunk: {}", text);
	}
	println!("");

	Ok(())
}
