use crate::settings::{self, AiSettings};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChatResult {
    pub model: String,
    pub response: String,
    pub endpoint: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: String,
}

pub fn chat(data_dir: &Path, prompt: String) -> Result<AiChatResult, String> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err("AI prompt cannot be empty".into());
    }
    if prompt.len() > 32_000 {
        return Err("AI prompt is limited to 32,000 characters".into());
    }

    let settings = settings::load(data_dir)?;
    if !settings.enabled {
        return Err("NekoAI is disabled. Enable it in the NekoAI page first.".into());
    }

    validate_endpoint_for_mode(&settings)?;
    let endpoint = chat_endpoint(&settings.endpoint);

    let body = json!({
        "model": settings.model,
        "messages": [
            {
                "role": "system",
                "content": "You are NekoAI inside NekoDroid. Respond concisely and do not claim to have controlled Android unless a separate validated control action was executed."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "stream": false
    });

    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Unable to create AI HTTP client: {e}"))?
        .post(&endpoint)
        .json(&body)
        .send()
        .map_err(|e| format!("AI request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("AI endpoint returned an error: {e}"))?;

    let parsed: ChatCompletionResponse = response
        .json()
        .map_err(|e| format!("AI endpoint returned an invalid chat-completions response: {e}"))?;

    let content = parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "AI endpoint returned no response text".to_string())?;

    append_log(data_dir, &settings, prompt, &content)?;

    Ok(AiChatResult {
        model: settings.model,
        response: content,
        endpoint,
    })
}

pub fn read_logs(data_dir: &Path) -> Result<String, String> {
    let path = data_dir.join("ai.log");
    if !path.exists() {
        return Ok(String::new());
    }
    let bytes = fs::read(&path).map_err(|e| format!("Unable to read AI log: {e}"))?;
    let start = bytes.len().saturating_sub(250_000);
    Ok(String::from_utf8_lossy(&bytes[start..]).to_string())
}

pub fn clear_logs(data_dir: &Path) -> Result<(), String> {
    let path = data_dir.join("ai.log");
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Unable to clear AI log: {e}"))?;
    }
    Ok(())
}

fn chat_endpoint(base: &str) -> String {
    let base = base.trim().trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    }
}

fn validate_endpoint_for_mode(settings: &AiSettings) -> Result<(), String> {
    let endpoint = settings.endpoint.trim();
    if !(endpoint.starts_with("http://") || endpoint.starts_with("https://")) {
        return Err("AI endpoint must use http:// or https://".into());
    }

    if settings.mode == "local"
        && !(endpoint.starts_with("http://127.0.0.1")
            || endpoint.starts_with("http://localhost")
            || endpoint.starts_with("https://127.0.0.1")
            || endpoint.starts_with("https://localhost"))
    {
        return Err("Local AI mode only accepts localhost endpoints".into());
    }

    Ok(())
}

fn append_log(
    data_dir: &Path,
    settings: &AiSettings,
    prompt: &str,
    response: &str,
) -> Result<(), String> {
    fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_dir.join("ai.log"))
        .map_err(|e| format!("Unable to open AI log: {e}"))?;

    let safe_prompt = prompt.replace('\r', " ").replace('\n', " ");
    let safe_response = response.replace('\r', " ").replace('\n', " ");
    writeln!(
        file,
        "[{timestamp}] mode={} model={} prompt={} response={}",
        settings.mode,
        settings.model,
        truncate(&safe_prompt, 1000),
        truncate(&safe_response, 4000)
    )
    .map_err(|e| format!("Unable to write AI log: {e}"))
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}
