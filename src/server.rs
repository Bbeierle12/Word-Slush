use axum::{extract::Json, http::StatusCode, response::IntoResponse, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use std::net::SocketAddr;

use crate::{counter, parser, tokenizer};

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub data: String,
    #[serde(default = "default_true")]
    pub normalize: bool,
    #[serde(default)]
    pub stop_words: bool,
    #[serde(default = "default_speaker")]
    pub speaker: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_true() -> bool { true }
fn default_speaker() -> String { "user".into() }
fn default_limit() -> usize { 100 }

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

async fn analyze(Json(req): Json<AnalyzeRequest>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let messages = parser::parse_json_str(&req.data)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Parse error: {e}")))?;

    let speaker = req.speaker.to_lowercase();
    let filtered: Vec<&parser::Message> = messages
        .iter()
        .filter(|m| match speaker.as_str() {
            "user" => m.role == "human",
            "assistant" => m.role == "assistant",
            _ => true,
        })
        .collect();

    let scoped_messages = filtered.len();

    let mut all_tokens = Vec::new();
    for msg in &filtered {
        all_tokens.extend(tokenizer::tokenize(&msg.content, req.normalize));
    }

    let counts = counter::count_words(all_tokens, req.stop_words);
    let total: u32 = counts.iter().map(|w| w.count).sum();
    let total_unique = counts.len();

    let display = if req.limit == 0 { &counts[..] } else { &counts[..req.limit.min(counts.len())] };

    let words: Vec<WordEntry> = display
        .iter()
        .enumerate()
        .map(|(i, wc)| WordEntry {
            rank: i + 1,
            word: wc.word.clone(),
            count: wc.count,
            percent: if total > 0 { (wc.count as f64 / total as f64) * 100.0 } else { 0.0 },
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
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("lexis server listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
