use std::{fs, str::FromStr};

use crate::{
    app_config::{AppConfig, Source},
    queue::{Queue, DB},
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use mime_guess::{mime, Mime};
use reqwest::{Client, Method, Request, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, Executor};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    approved_at_utc: Option<serde_json::Value>,
    subreddit: Option<String>,
    selftext: Option<String>,
    author_fullname: Option<String>,
    saved: bool,
    mod_reason_title: Option<serde_json::Value>,
    gilded: Option<f64>,
    clicked: bool,
    title: String,
    link_flair_richtext: Option<Vec<Option<serde_json::Value>>>,
    subreddit_name_prefixed: Option<String>,
    hidden: bool,
    pwls: Option<f64>,
    link_flair_css_class: Option<serde_json::Value>,
    downs: Option<f64>,
    thumbnail_height: Option<serde_json::Value>,
    top_awarded_type: Option<serde_json::Value>,
    hide_score: bool,
    name: Option<String>,
    quarantine: bool,
    link_flair_text_color: Option<String>,
    upvote_ratio: Option<f64>,
    author_flair_background_color: Option<serde_json::Value>,
    subreddit_type: Option<String>,
    ups: Option<f64>,
    total_awards_received: Option<f64>,
    thumbnail_width: Option<serde_json::Value>,
    author_flair_template_id: Option<String>,
    is_original_content: bool,
    user_reports: Option<Vec<Option<serde_json::Value>>>,
    secure_media: Option<serde_json::Value>,
    is_reddit_media_domain: bool,
    is_meta: bool,
    category: Option<serde_json::Value>,
    link_flair_text: Option<serde_json::Value>,
    can_mod_post: bool,
    score: Option<f64>,
    approved_by: Option<serde_json::Value>,
    is_created_from_ads_ui: bool,
    author_premium: bool,
    thumbnail: Option<String>,
    edited: bool,
    author_flair_css_class: Option<String>,
    author_flair_richtext: Option<Vec<Option<serde_json::Value>>>,
    content_categories: Option<Vec<String>>,
    is_self: bool,
    mod_note: Option<serde_json::Value>,
    created: Option<f64>,
    link_flair_type: Option<String>,
    wls: Option<f64>,
    removed_by_category: Option<serde_json::Value>,
    banned_by: Option<serde_json::Value>,
    author_flair_type: Option<String>,
    domain: Option<String>,
    allow_live_comments: bool,
    selftext_html: Option<serde_json::Value>,
    likes: Option<serde_json::Value>,
    suggested_sort: Option<serde_json::Value>,
    banned_at_utc: Option<serde_json::Value>,
    view_count: Option<serde_json::Value>,
    archived: bool,
    no_follow: bool,
    is_crosspostable: bool,
    pinned: bool,
    over_18: bool,
    awarders: Option<Vec<Option<serde_json::Value>>>,
    media_only: bool,
    can_gild: bool,
    spoiler: bool,
    locked: bool,
    author_flair_text: Option<String>,
    treatment_tags: Option<Vec<Option<serde_json::Value>>>,
    visited: bool,
    removed_by: Option<serde_json::Value>,
    num_reports: Option<serde_json::Value>,
    distinguished: Option<serde_json::Value>,
    subreddit_id: Option<String>,
    author_is_blocked: bool,
    mod_reason_by: Option<serde_json::Value>,
    removal_reason: Option<serde_json::Value>,
    link_flair_background_color: Option<String>,
    id: String,
    is_robot_indexable: bool,
    report_reasons: Option<serde_json::Value>,
    author: Option<String>,
    discussion_type: Option<serde_json::Value>,
    num_comments: Option<f64>,
    send_replies: bool,
    whitelist_status: Option<String>,
    contest_mode: bool,
    mod_reports: Option<Vec<Option<serde_json::Value>>>,
    author_patreon_flair: bool,
    author_flair_text_color: Option<String>,
    permalink: String,
    parent_whitelist_status: Option<String>,
    stickied: bool,
    url: String,
    subreddit_subscribers: Option<f64>,
    created_utc: Option<f64>,
    num_crossposts: Option<f64>,
    media: Option<serde_json::Value>,
    is_video: bool,
}

pub async fn get_from_subreddit(
    name: &str,
    config: &AppConfig,
    app_handle: &AppHandle,
) -> Result<()> {
    let mut last_post: Option<Post> = None;
    let mut posts: Vec<Post> = Vec::new();
    let url = format!("https://www.reddit.com/r/{name}/hot.json");
    while last_post.is_none() {
        let mut url = Url::parse(url.as_str())?;
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("client_id", "8Msi7s37PSEWqkB9G-otfQ");
            if let Some(post) = last_post {
                query_pairs.append_pair("after", post.id.as_str());
            };
        }
        let res = Client::new()
            .execute(Request::new(Method::GET, url))
            .await?
            .text()
            .await?;
        let json: Value = serde_json::from_str(&res)?;
        let queue = app_handle.state::<Queue>().lock().await.clone();
        let new_posts: Vec<Post> = json
            .get("data")
            .ok_or(anyhow!("Data not found"))?
            .get("children")
            .ok_or(anyhow!("Children not found"))?
            .as_array()
            .ok_or(anyhow!("No posts array"))?
            .iter()
            .filter_map(|data| {
                let post =
                    serde_json::from_value::<Option<Post>>(data.get("data")?.to_owned()).ok()??;
                if !post.is_self
                    && !post.is_meta
                    && (config.allow_nsfw || !post.over_18)
                    && mime_guess::from_path(&post.url)
                        .iter()
                        .any(|m| m.type_() == mime::IMAGE)
                    && queue
                        .iter()
                        .find(|&wp| wp.id.to_owned() == post.id)
                        .is_none()
                {
                    Some(post)
                } else {
                    None
                }
            })
            .collect();
        last_post = new_posts.first().map(|p| p.clone());
        posts = new_posts;
    }
    let source_str = serde_json::to_string(&Source::Subreddit(name.to_string()))?;
    let mut transaction = app_handle.state::<DB>().begin().await?;

    match {
        for post in posts {
            let wp_res = reqwest::get(&post.url).await?;
            let wallpaper_filename = format!(
                "{}.{}",
                post.id,
                Mime::from_str(
                    wp_res
                        .headers()
                        .get("Content-Type")
                        .ok_or(anyhow!("Couldn't determine extension"))?
                        .to_str()?
                )?
                .subtype()
                .as_str()
            );
            fs::write(&wallpaper_filename, wp_res.bytes().await?)?;
            let now = Utc::now();
            transaction
                .execute(query!(
            "INSERT INTO queue (id, info_url, file_name, source, date) VALUES ($1, $2, $3, $4, $5)",
            post.id,
            post.url,
            wallpaper_filename,
            source_str,
            now
        ))
                .await?;
        }
        Ok(())
    } {
        Ok(()) => transaction.commit().await.map_err(|e| e.into()),
        Err(e) => {
            transaction.rollback().await?;
            return Err(e);
        }
    }
}
