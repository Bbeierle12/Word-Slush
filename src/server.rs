use axum::extract::{DefaultBodyLimit, Multipart};
use axum::{http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use serde::Serialize;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use std::net::SocketAddr;

use crate::{counter, parser, tokenizer};

#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub words: Vec<WordEntry>,
    pub total_words: u32,
    pub total_unique: usize,
    pub scoped_messages: usize,
}

#[derive(Serialize)]
pub struct WordEntry {
    pub rank: usize,
    pub word: String,
    pub count: u32,
    pub percent: f64,
}

async fn analyze(mut multipart: Multipart) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut normalize = true;
    let mut stop_words = false;
    let mut speaker = "user".to_string();
    let mut limit: usize = 100;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Read error: {e}")))?
                        .to_vec(),
                );
            }
            "normalize" => {
                let v = field.text().await.unwrap_or_default();
                normalize = v != "false";
            }
            "stop_words" => {
                let v = field.text().await.unwrap_or_default();
                stop_words = v == "true";
            }
            "speaker" => {
                speaker = field.text().await.unwrap_or_else(|_| "user".into());
            }
            "limit" => {
                let v = field.text().await.unwrap_or_default();
                limit = v.parse().unwrap_or(100);
            }
            _ => {}
        }
    }

    let bytes = file_bytes
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'file' field".into()))?;
    let fname = file_name.unwrap_or_else(|| "upload.txt".into());

    let messages = parser::parse_bytes(&fname, &bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Parse error: {e}")))?;

    let speaker_lower = speaker.to_lowercase();
    let filtered: Vec<&parser::Message> = messages
        .iter()
        .filter(|m| match speaker_lower.as_str() {
            "user" => m.role == "human",
            "assistant" => m.role == "assistant",
            _ => true,
        })
        .collect();

    let scoped_messages = filtered.len();

    let mut all_tokens = Vec::new();
    for msg in &filtered {
        all_tokens.extend(tokenizer::tokenize(&msg.content, normalize));
    }

    let counts = counter::count_words(all_tokens, stop_words);
    let total: u32 = counts.iter().map(|w| w.count).sum();
    let total_unique = counts.len();

    let display = if limit == 0 {
        &counts[..]
    } else {
        &counts[..limit.min(counts.len())]
    };

    let words: Vec<WordEntry> = display
        .iter()
        .enumerate()
        .map(|(i, wc)| WordEntry {
            rank: i + 1,
            word: wc.word.clone(),
            count: wc.count,
            percent: if total > 0 {
                (wc.count as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    Ok(Json(AnalyzeResponse {
        words,
        total_words: total,
        total_unique,
        scoped_messages,
    }))
}

async fn health() -> &'static str {
    "ok"
}

pub async fn run(port: u16) {
    let ui_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("ui")))
        .unwrap_or_else(|| std::path::PathBuf::from("ui/dist"));

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/analyze", post(analyze))
        .fallback_service(ServeDir::new(&ui_path).append_index_html_on_directories(true))
        .layer(DefaultBodyLimit::max(512 * 1024 * 1024)) // 512 MB
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("lexis server listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
