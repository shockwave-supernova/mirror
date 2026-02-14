use dotenv::dotenv;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use reqwest::{Client, multipart};
use regex::Regex;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
struct Status {
    id: String,
    content: String,
    visibility: String,
    in_reply_to_id: Option<String>,
    media_attachments: Vec<Media>,
    reblog: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
struct Media {
    url: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct PostStatus<'a> {
    status: String,
    visibility: &'a str,
    media_ids: Vec<String>,
}

/// Cleans up HTML tags from Mastodon posts and decodes HTML entities
fn clean_html(html: &str) -> String {
    let mut text = html.replace("<br />", "\n").replace("<br>", "\n").replace("</p><p>", "\n\n");
    let re = Regex::new(r"<[^>]*>").unwrap();
    text = re.replace_all(&text, "").to_string();
    text = text.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'");
    text.trim().to_string()
}

/// Downloads media from the source URL and uploads it to the target instance
async fn upload_media(client: &Client, url: &str, desc: Option<String>, tgt_url: &str, tgt_token: &str) -> Result<String> {
    // Download file
    let resp = client.get(url).send().await?.bytes().await?;
    let file_name = url.split('/').last().unwrap_or("file.jpg").to_string();

    let part = multipart::Part::bytes(resp.to_vec())
        .file_name(file_name)
        .mime_str("application/octet-stream")?;

    let mut form = multipart::Form::new().part("file", part);
    if let Some(d) = desc { form = form.text("description", d); }

    // Upload to target
    let res = client.post(format!("{}/api/v2/media", tgt_url))
        .header("Authorization", format!("Bearer {}", tgt_token))
        .multipart(form).send().await?;

    if res.status().is_success() {
        let json: serde_json::Value = res.json().await?;
        Ok(json["id"].as_str().unwrap().to_string())
    } else {
        // Error handling with attitude
        anyhow::bail!("🚫 Ugh, media upload failed! My vibe is ruined. Status: {}", res.status())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();
    let src_url = env::var("SOURCE_URL")?;
    let src_token = env::var("SOURCE_TOKEN")?;
    let tgt_url = env::var("TARGET_URL")?;
    let tgt_token = env::var("TARGET_TOKEN")?;

    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;

    // Verify credentials and fetch current user ID
    let me: serde_json::Value = client.get(format!("{}/api/v1/accounts/verify_credentials", src_url))
        .header("Authorization", format!("Bearer {}", src_token)).send().await?.json().await?;
    let my_id = me["id"].as_str().unwrap().to_string();

    println!("✨ Makeup ready! Mirror Queen activated! Stalking account: {} 💅", me["username"]);

    // Fetch the most recent post ID to establish a baseline and avoid duplicating old history
    let mut last_id = client.get(format!("{}/api/v1/accounts/{}/statuses?limit=1", src_url, my_id))
        .header("Authorization", format!("Bearer {}", src_token)).send().await?
        .json::<Vec<Status>>().await?.first().map(|s| s.id.clone()).unwrap_or_default();

    println!("🔎 Found the latest tea spill (ID: {}). Waiting for fresh drama... ☕", last_id);

    // Main polling loop
    loop {
        let url = format!("{}/api/v1/accounts/{}/statuses?since_id={}", src_url, my_id, last_id);
        let resp = client.get(url)
            .header("Authorization", format!("Bearer {}", src_token))
            .send().await;

        if let Ok(r) = resp {
            if let Ok(mut statuses) = r.json::<Vec<Status>>().await {
                // Process posts from oldest to newest
                statuses.reverse();

                for s in statuses {
                    // Skip reblogs and replies to keep the feed clean
                    if s.reblog.is_some() || s.in_reply_to_id.is_some() {
                        last_id = s.id.clone();
                        continue;
                    }

                    // Clean HTML content
                    let text = clean_html(&s.content);

                    // Skip direct mentions starting with @
                    if text.starts_with('@') {
                        last_id = s.id.clone();
                        continue;
                    }

                    println!("💌 Ooh, fresh content incoming! (ID: {}): '{}...' Stealing it! 💖", s.id, text.chars().take(30).collect::<String>());

                    // Process attachments
                    let mut media_ids = Vec::new();
                    for m in s.media_attachments {
                        match upload_media(&client, &m.url, m.description, &tgt_url, &tgt_token).await {
                            Ok(mid) => media_ids.push(mid),
                            Err(_) => println!("⚠️ Oopsie, couldn't upload a pic. Whatever, posting without it. 🙄"),
                        }
                    }

                    // Post to target instance
                    let params = PostStatus { status: text, visibility: "private", media_ids };
                    let post_res = client.post(format!("{}/api/v1/statuses", tgt_url))
                        .header("Authorization", format!("Bearer {}", tgt_token))
                        .json(&params).send().await;

                    if let Ok(pr) = post_res {
                        if pr.status().is_success() {
                            println!("🎉 Posted! I'm literally the best bot ever. ✨");
                            last_id = s.id.clone();
                        } else if pr.status() == 429 {
                            // Handle rate limiting
                            println!("🛑 Ugh, rate limit! Too much attention. Taking a 5-min beauty nap. Don't disturb! 😴");
                            sleep(Duration::from_secs(300)).await;
                        }
                    }

                    // Short delay between posts to be polite
                    sleep(Duration::from_secs(10)).await;
                }
            }
        }

        // Poll interval (every 2 minutes)
        sleep(Duration::from_secs(120)).await;
    }
}
