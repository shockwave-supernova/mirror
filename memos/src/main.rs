use dotenvy::dotenv;
use html2md::parse_html;
use megalodon::{generator, megalodon::SearchInputOptions};
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initializing environment variables from .env file
    dotenv().ok();

    let m_url = env::var("MASTODON_URL").expect("MASTODON_URL missing");
    let m_token = env::var("MASTODON_TOKEN").expect("MASTODON_TOKEN missing");
    let memos_url = env::var("MEMOS_URL").expect("MEMOS_URL missing");
    let memos_token = env::var("MEMOS_TOKEN").expect("MEMOS_TOKEN missing");

    println!("==================================================");
    println!("🚀 MASTODON TO MEMOS");
    println!("==================================================");

    // Initializing Mastodon client
    let action_client = generator(
        megalodon::SNS::Mastodon,
        m_url.clone(),
        Some(m_token.clone()),
        None,
    ).expect("Failed to create Mastodon client");

    let http_client = Client::new();
    let status_url_regex = Regex::new(r#"https?://[^\s<"]+"#).unwrap();

    let me = action_client.verify_account_credentials().await?.json;
    println!("👤 Connected as: {} (ID: {})", me.username, me.id);

    let feed_endpoint = format!("{}/api/v1/accounts/{}/statuses?limit=5", m_url, me.id);

    loop {
        // Fetching timeline manually
        let resp = http_client.get(&feed_endpoint)
            .header("Authorization", format!("Bearer {}", m_token))
            .send()
            .await;

        if let Ok(response) = resp {
            if response.status().is_success() {
                let statuses: Vec<Value> = response.json().await.unwrap_or_default();

                for status in statuses {
                    let status_id = status["id"].as_str().unwrap_or_default().to_string();

                    let has_memos_tag = status["tags"].as_array()
                        .map(|tags| tags.iter().any(|t| t["name"].as_str().unwrap_or("").to_lowercase() == "memos"))
                        .unwrap_or(false);

                    if has_memos_tag {
                        println!("\n🎯 Found trigger post: {}", status_id);

                        // Added 'author_username' to the extraction
                        let (content_html, source_url, media_links, poll_info, author_header, author_username) =
                            extract_full_data(&status, &*action_client, &status_url_regex).await;

                        let mut markdown = parse_html(&content_html);

                        markdown = markdown.replace("\\#", "#").replace("\\[", "[").replace("\\]", "]")
                            .replace("\\*", "*").replace("\\_", "_").replace("\\>", ">").replace("\\`", "`");

                        let mut final_payload = format!("{}\n\n", author_header);

                        final_payload.push_str(&markdown);

                        if !poll_info.is_empty() {
                            final_payload.push_str("\n\n📊 **Poll Options:**");
                            final_payload.push_str(&poll_info);
                        }

                        if !media_links.is_empty() {
                            final_payload.push_str("\n\n🖼️ **Media:**\n");
                            final_payload.push_str(&media_links);
                        }

                        if let Some(url) = source_url {
                            final_payload.push_str(&format!("\n\n🔗 **Source:** {}", url));
                        }

                        // DYNAMIC HASHTAG GENERATION
                        // Adds #mastodon #mastodon2memos AND #{username}
                        final_payload.push_str(&format!("\n\n#mastodon #mastodon2memos #{}", author_username));

                        println!("📦 Sending payload to Memos...");
                        let memos_res = http_client.post(format!("{}/api/v1/memos", memos_url))
                            .header("Authorization", format!("Bearer {}", memos_token))
                            .json(&json!({
                                "content": final_payload,
                                "visibility": "PRIVATE"
                            }))
                            .send()
                            .await;

                        if let Ok(m_resp) = memos_res {
                            if m_resp.status().is_success() {
                                println!("✅ Sync successful!");

                                let _ = if !status["reblog"].is_null() {
                                    action_client.unreblog_status(status_id).await.map(|_| ())
                                } else {
                                    action_client.delete_status(status_id).await.map(|_| ())
                                };
                                break;
                            }
                        }
                    }
                }
            }
        }
        sleep(Duration::from_secs(60)).await;
    }
}

// Updated signature to return author_username (String) at the end
async fn extract_full_data(
    status: &Value,
    client: &dyn megalodon::Megalodon,
    regex: &Regex
) -> (String, Option<String>, String, String, String, String) {
    let mut target_status = status;

    if !status["reblog"].is_null() {
        target_status = &status["reblog"];
    } else {
        let content = status["content"].as_str().unwrap_or("");
        if let Some(mat) = regex.find(content) {
            let url = mat.as_str();
            if url.contains("/@") && url.chars().any(|c| c.is_numeric()) {
                let opts = SearchInputOptions { resolve: Some(true), ..Default::default() };
                if let Ok(res) = client.search(url.to_string(), Some(&opts)).await {
                    if let Some(s) = res.json.statuses.first() {
                        let media = s.media_attachments.iter()
                            .map(|m| format!("![]({})", m.url))
                            .collect::<Vec<_>>()
                            .join("\n");

                        let header = format!(
                            "<img src=\"{}\" width=\"32\" height=\"32\" style=\"border-radius:4px; vertical-align:middle; display:inline-block; margin:0 8px 0 0;\"><span style=\"vertical-align:middle;\">**{}** (@{})</span>",
                            s.account.avatar, s.account.display_name, s.account.username
                        );

                        // Return username as the last element
                        return (s.content.clone(), s.url.clone(), media, String::new(), header, s.account.username.clone());
                    }
                }
            }
        }
    }

    let html = target_status["content"].as_str().unwrap_or("").to_string();
    let url = target_status["url"].as_str().map(|s| s.to_string());

    let author_name = target_status["account"]["display_name"].as_str().unwrap_or("Unknown");
    let author_handle = target_status["account"]["username"].as_str().unwrap_or("unknown"); // This is the pure username
    let author_avatar = target_status["account"]["avatar"].as_str().unwrap_or("");

    let author_header = format!(
        "<img src=\"{}\" width=\"32\" height=\"32\" style=\"border-radius:4px; vertical-align:middle; display:inline-block; margin:0 8px 0 0;\"><span style=\"vertical-align:middle;\">**{}** (@{})</span>",
        author_avatar, author_name, author_handle
    );

    let mut media_links = String::new();
    if let Some(attachments) = target_status["media_attachments"].as_array() {
        for anim in attachments {
            if let Some(m_url) = anim["url"].as_str() {
                media_links.push_str(&format!("![]({})\n", m_url));
            }
        }
    }

    let mut poll_info = String::new();
    if !target_status["poll"].is_null() {
        if let Some(options) = target_status["poll"]["options"].as_array() {
            for opt in options {
                let title = opt["title"].as_str().unwrap_or("");
                let votes = opt["votes_count"].as_u64().unwrap_or(0);
                poll_info.push_str(&format!("\n* {} ({} votes)", title, votes));
            }
        }
    }

    // Return author_handle (string) as the last element
    (html, url, media_links, poll_info, author_header, author_handle.to_string())
}
