use std::env;
use dotenv::dotenv;
use reqwest::blocking::{Client, multipart};
use serde::Deserialize;

/// Represents the top-level response structure from the Wallhaven API.
#[derive(Deserialize, Debug)]
struct WallhavenResponse {
    data: Vec<WallhavenPhoto>,
}

/// Represents individual photo metadata from Wallhaven.
#[derive(Deserialize, Debug)]
struct WallhavenPhoto {
    path: String,
    dimension_x: u32,
    dimension_y: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment variables from .env file
    dotenv().ok();

    let wallhaven_key = env::var("WALLHAVEN_API_KEY")
        .expect("Environment variable WALLHAVEN_API_KEY is required");
    let masto_token = env::var("MASTODON_ACCESS_TOKEN")
        .expect("Environment variable MASTODON_ACCESS_TOKEN is required");
    let masto_url = env::var("MASTODON_INSTANCE_URL")
        .expect("Environment variable MASTODON_INSTANCE_URL is required");

    // Initialize HTTP client with a custom User-Agent to comply with API policies
    let client = Client::builder()
        .user_agent("WallhavenHeaderAutomation/1.0 (Rust; Cross-Compiled)")
        .build()?;

    // Step 1: Query Wallhaven API for high-resolution cyberpunk imagery
    // Constraints:
    // - Search: "cyberpunk city"
    // - Sorting: Random to ensure daily variety
    // - Dimensions: Minimum 1500x500 (Mastodon header specification)
    // - Ratios: Wide formats (16x9, 32x9) to prevent aggressive cropping
    let search_url = format!(
        "https://wallhaven.cc/api/v1/search?apikey={}&q=cyberpunk+city&sorting=random&atleast=1500x500&ratios=16x9,32x9,21x9",
        wallhaven_key
    );

    println!("[INFO] Querying Wallhaven API...");
    let search_resp = client.get(search_url).send()?.json::<WallhavenResponse>()?;

    if let Some(photo) = search_resp.data.first() {
        println!("[INFO] Target image identified: {} ({}x{})", photo.path, photo.dimension_x, photo.dimension_y);

        // Step 2: Download the selected resource into memory
        println!("[INFO] Fetching image bytes...");
        let img_bytes = client.get(&photo.path).send()?.bytes()?;

        // Step 3: Update Mastodon account credentials
        // Endpoint: PATCH /api/v1/accounts/update_credentials
        // Documentation: https://docs.joinmastodon.org/methods/accounts/#update_credentials
        println!("[INFO] Updating Mastodon profile header...");
        let update_url = format!("{}/api/v1/accounts/update_credentials", masto_url);

        let image_part = multipart::Part::bytes(img_bytes.to_vec())
            .file_name("header.jpg")
            .mime_str("image/jpeg")?;

        let form = multipart::Form::new()
            .part("header", image_part);

        let masto_resp = client.patch(&update_url)
            .header("Authorization", format!("Bearer {}", masto_token))
            .multipart(form)
            .send()?;

        if masto_resp.status().is_success() {
            println!("[SUCCESS] Profile header successfully updated.");
        } else {
            let error_text = masto_resp.text()?;
            eprintln!("[ERROR] Mastodon API rejection: {}", error_text);
        }
    } else {
        println!("[WARN] No images matching the specified criteria were found.");
    }

    Ok(())
}
