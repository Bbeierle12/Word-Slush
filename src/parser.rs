use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Conversation {
    pub messages: Vec<Message>,
}

pub fn parse_export(path: &Path) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    parse_json_str(&data)
}

pub fn parse_json_str(data: &str) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let raw: serde_json::Value = serde_json::from_str(data)?;

    // Support both a single conversation object and an array of conversations
    let messages = if raw.as_array().is_some() {
        let convos: Vec<Conversation> = serde_json::from_value(raw.clone())?;
        convos.into_iter().flat_map(|c| c.messages).collect()
    } else if raw.get("messages").is_some() {
        let convo: Conversation = serde_json::from_value(raw)?;
        convo.messages
    } else {
        return Err("Unrecognized JSON format: expected object with 'messages' or array of such objects".into());
    };

    Ok(messages)
}
