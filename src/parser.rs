use serde::Deserialize;
use std::io::Read as IoRead;
use std::path::Path;

/// Normalized message with role and plain-text content.
pub struct Message {
    pub role: String,
    pub content: String,
}

// --- Flexible deserialization for Claude's varying export formats ---

/// Content can be a plain string or an array of content blocks.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Content {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    text: Option<String>,
    // tool_use, image, etc. — we extract text where available
    #[serde(default)]
    input: Option<serde_json::Value>,
}

impl Content {
    fn into_text(self) -> String {
        match self {
            Content::Text(s) => s,
            Content::Blocks(blocks) => {
                let mut parts = Vec::new();
                for block in blocks {
                    if let Some(text) = block.text {
                        parts.push(text);
                    }
                    // For tool_use blocks, extract input as stringified JSON
                    if block.r#type == "tool_use" {
                        if let Some(input) = block.input {
                            if let Ok(s) = serde_json::to_string(&input) {
                                parts.push(s);
                            }
                        }
                    }
                }
                parts.join(" ")
            }
        }
    }
}

/// Raw message as it appears in the export JSON.
#[derive(Debug, Deserialize)]
struct RawMessage {
    #[serde(alias = "role")]
    role: String,
    content: Content,
}

impl RawMessage {
    fn into_message(self) -> Message {
        // Normalize role: Claude API uses "user", web UI uses "human"
        let role = match self.role.to_lowercase().as_str() {
            "user" | "human" => "human".to_string(),
            "assistant" => "assistant".to_string(),
            "system" => "system".to_string(),
            other => other.to_string(),
        };
        Message {
            role,
            content: self.content.into_text(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Conversation {
    messages: Vec<RawMessage>,
}

/// Wrapper format: `{ "conversations": [...] }`
#[derive(Debug, Deserialize)]
struct ConversationWrapper {
    conversations: Vec<Conversation>,
}

/// Entry point for CLI — reads file from disk, dispatches by type.
pub fn parse_export(path: &Path) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let data = std::fs::read(path)?;
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file.txt");
    parse_bytes(filename, &data)
}

/// Main dispatcher — routes by file extension.
pub fn parse_bytes(filename: &str, data: &[u8]) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "json" => {
            let text = String::from_utf8_lossy(data);
            match parse_json_str(&text) {
                Ok(msgs) if !msgs.is_empty() => Ok(msgs),
                Ok(_) => Ok(wrap_plain(text.into_owned())),
                Err(_) => Ok(wrap_plain(text.into_owned())),
            }
        }
        "zip" => parse_zip(data),
        _ => {
            let text = String::from_utf8_lossy(data).into_owned();
            Ok(wrap_plain(text))
        }
    }
}

/// Parse Claude conversation JSON — supports multiple formats:
/// - `{ "messages": [...] }` (single conversation)
/// - `[{ "messages": [...] }, ...]` (array of conversations)
/// - `{ "conversations": [{ "messages": [...] }] }` (nested wrapper)
pub fn parse_json_str(data: &str) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let raw: serde_json::Value = serde_json::from_str(data)?;

    // Try array of conversations
    if raw.as_array().is_some() {
        if let Ok(convos) = serde_json::from_value::<Vec<Conversation>>(raw.clone()) {
            let msgs: Vec<Message> = convos
                .into_iter()
                .flat_map(|c| c.messages.into_iter().map(|m| m.into_message()))
                .collect();
            if !msgs.is_empty() {
                return Ok(msgs);
            }
        }
    }

    // Try single conversation with "messages" field
    if raw.get("messages").is_some() {
        if let Ok(convo) = serde_json::from_value::<Conversation>(raw.clone()) {
            let msgs: Vec<Message> = convo.messages.into_iter().map(|m| m.into_message()).collect();
            if !msgs.is_empty() {
                return Ok(msgs);
            }
        }
    }

    // Try nested wrapper: { "conversations": [...] }
    if raw.get("conversations").is_some() {
        if let Ok(wrapper) = serde_json::from_value::<ConversationWrapper>(raw.clone()) {
            let msgs: Vec<Message> = wrapper
                .conversations
                .into_iter()
                .flat_map(|c| c.messages.into_iter().map(|m| m.into_message()))
                .collect();
            if !msgs.is_empty() {
                return Ok(msgs);
            }
        }
    }

    Err("Unrecognized JSON format: expected conversation object(s) with 'messages' field".into())
}

/// Wrap plain text as a single human message so default speaker filter includes it.
fn wrap_plain(content: String) -> Vec<Message> {
    vec![Message {
        role: "human".into(),
        content,
    }]
}

/// Extract zip archive, recursively process each entry.
fn parse_zip(data: &[u8]) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    parse_zip_inner(data, 0)
}

const MAX_ZIP_DEPTH: usize = 5;

fn parse_zip_inner(data: &[u8], depth: usize) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    if depth > MAX_ZIP_DEPTH {
        return Err("Zip nesting too deep (max 5 levels)".into());
    }

    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    let mut all_messages = Vec::new();
    let mut skipped = Vec::new();
    let total_entries = archive.len();

    for i in 0..total_entries {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();

        // Skip entries with path traversal
        if name.contains("..") {
            skipped.push(name);
            continue;
        }

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;

        // Check if it's a nested zip
        let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
        if ext == "zip" {
            match parse_zip_inner(&buf, depth + 1) {
                Ok(msgs) => all_messages.extend(msgs),
                Err(_) => skipped.push(name),
            }
        } else {
            match parse_bytes(&name, &buf) {
                Ok(msgs) => all_messages.extend(msgs),
                Err(_) => skipped.push(name),
            }
        }
    }

    if !skipped.is_empty() {
        eprintln!(
            "lexis: skipped {} file(s) in zip: {}",
            skipped.len(),
            skipped.join(", ")
        );
    }

    if all_messages.is_empty() {
        return Err(format!(
            "Zip archive contained no processable text files ({total_entries} entries, {} skipped)",
            skipped.len()
        )
        .into());
    }
    Ok(all_messages)
}
