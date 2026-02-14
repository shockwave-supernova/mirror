use std::collections::HashSet;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::thread;
use std::time::Duration;
use dotenv::dotenv;
use reqwest::blocking::Client;
use reqwest::blocking::multipart;
use serde::Deserialize;

// --- Unsplash Structs ---

#[derive(Deserialize, Debug)]
struct UnsplashPhoto {
    id: String,
    description: Option<String>,
    alt_description: Option<String>,
    urls: UnsplashUrls,
    links: UnsplashLinks,
    user: UnsplashUser,
    tags: Option<Vec<UnsplashTag>>,
    location: Option<UnsplashLocation>,
}

#[derive(Deserialize, Debug)]
struct UnsplashLocation {
    name: Option<String>,
    city: Option<String>,
    country: Option<String>,
}

#[derive(Deserialize, Debug)]
struct UnsplashUrls {
    regular: String,
}

#[derive(Deserialize, Debug)]
struct UnsplashLinks {
    html: String,
}

#[derive(Deserialize, Debug)]
struct UnsplashUser {
    name: String,
}

#[derive(Deserialize, Debug)]
struct UnsplashTag {
    title: String,
}

// --- Mastodon Structs ---

#[derive(Deserialize, Debug)]
struct MastoMedia {
    id: String,
}

// --- History Logic ---

const HISTORY_FILE: &str = "history.json";

fn load_history() -> HashSet<String> {
    if let Ok(file) = File::open(HISTORY_FILE) {
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_default()
    } else {
        HashSet::new()
    }
}

fn save_history(history: &HashSet<String>) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(HISTORY_FILE)
        .expect("Failed to open history file");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, history).expect("Failed to save history");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load environment variables
    let unsplash_key = env::var("UNSPLASH_ACCESS_KEY").expect("Missing UNSPLASH_ACCESS_KEY");
    let masto_token = env::var("MASTODON_ACCESS_TOKEN").expect("Missing MASTODON_ACCESS_TOKEN");
    let masto_url = env::var("MASTODON_INSTANCE_URL").expect("Missing MASTODON_INSTANCE_URL");

    let client = Client::new();
    let mut history = load_history();

    // 1. Search for photos (Topic: Wallpapers, Order: Popular)
    let url = "https://api.unsplash.com/topics/wallpapers/photos?orientation=landscape&order_by=popular&page=1&per_page=30";

    println!("--> [1/4] Searching Unsplash...");
    let resp = client.get(url)
        .header("Authorization", format!("Client-ID {}", unsplash_key))
        .send()?
        .json::<Vec<UnsplashPhoto>>()?;

    // Select the first photo that is not in the history
    let selected_photo = resp.iter().find(|p| !history.contains(&p.id));

    if let Some(photo) = selected_photo {
        println!("--> Selected Photo: {}", photo.id);

        // --- Build ALT TEXT ---

        // A. Location
        let location_str = if let Some(loc) = &photo.location {
            if let Some(name) = &loc.name {
                format!("📍 Location: {}\n", name)
            } else if let (Some(city), Some(country)) = (&loc.city, &loc.country) {
                format!("📍 Location: {}, {}\n", city, country)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // B. Description
        let desc_str = photo.description.as_ref()
            .or(photo.alt_description.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Wallpaper");

        // C. Tags (take up to 15)
        let tags_str = photo.tags.as_ref().map(|tags| {
            tags.iter()
                .take(15)
                .map(|t| format!("#{}", t.title.replace(" ", "").replace("-", "")))
                .collect::<Vec<String>>()
                .join(" ")
        }).unwrap_or_default();

        // D. Credits
        let credit_str = format!("📷 Photo by {}\n🔗 Original: {}", photo.user.name, photo.links.html);

        // Combine everything into one string for Alt Text
        let full_alt_text = format!(
            "{}\n{}\n\nTags: {}\n\n---\n{}",
            location_str, desc_str, tags_str, credit_str
        );

        // Truncate if it exceeds 1500 characters (Mastodon limit)
        let safe_alt_text = if full_alt_text.len() > 1490 {
            full_alt_text[..1490].to_string()
        } else {
            full_alt_text
        };

        // --- Download and Upload ---

        println!("--> [2/4] Downloading image...");
        let img_bytes = client.get(&photo.urls.regular).send()?.bytes()?;

        println!("--> [3/4] Uploading to Mastodon...");
        let upload_url = format!("{}/api/v2/media", masto_url);

        let part = multipart::Part::bytes(img_bytes.to_vec())
            .file_name("wallpaper.jpg")
            .mime_str("image/jpeg")?;

        // Important: the 'description' field here sets the Alt Text
        let form = multipart::Form::new()
            .part("file", part)
            .text("description", safe_alt_text);

        let media_resp = client.post(&upload_url)
            .header("Authorization", format!("Bearer {}", masto_token))
            .multipart(form)
            .send()?;

        if !media_resp.status().is_success() {
            println!("Error Uploading Media: {:?}", media_resp.text());
            return Ok(());
        }

        let media_json: MastoMedia = media_resp.json()?;
        println!("    Media ID: {}", media_json.id);

        // Give the server some time to process the uploaded image
        thread::sleep(Duration::from_secs(3));

        // --- Posting ---

        println!("--> [4/4] Posting clean status...");

        let post_url = format!("{}/api/v1/statuses", masto_url);

        // Send an empty status body, but with attached media_ids
        let post_params = serde_json::json!({
            "status": "",
            "media_ids": [media_json.id],
            "visibility": "public"
        });

        let post_resp = client.post(&post_url)
            .header("Authorization", format!("Bearer {}", masto_token))
            .json(&post_params)
            .send()?;

        if post_resp.status().is_success() {
            println!("--> SUCCESS!");
            history.insert(photo.id.clone());
            save_history(&history);
        } else {
            println!("Error Posting Status: {:?}", post_resp.text());
        }

    } else {
        println!("No new photos found on the first page.");
    }

    Ok(())
}
