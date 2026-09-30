use crate::{adb, models::AdbResult, settings::{self, AiSettings}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AiAction {
    Tap { x: i32, y: i32 },
    Hold { x: i32, y: i32, duration_ms: u32 },
    Swipe { x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32 },
    Drag { x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32 },
    Key { keycode: String },
    Text { text: String },
}

static ACTION_TIMES: OnceLock<Mutex<VecDeque<Instant>>> = OnceLock::new();
static CANCEL_ACTIONS: AtomicBool = AtomicBool::new(false);

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
                "content": format!("You are NekoAI inside NekoDroid. Respond concisely and do not claim to have controlled Android unless a separate validated control action was executed. Current helper mode: {}. Inventory mode focuses on inventory information, quest mode on objectives, ui mode on interface assistance, and repetitive-task mode on clearly user-authorized repeated UI tasks.", settings.helper_mode)
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

    let safe_prompt = redact_sensitive(&prompt.replace('\r', " ").replace('\n', " "));
    let safe_response = redact_sensitive(&response.replace('\r', " ").replace('\n', " "));
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

fn redact_sensitive(value: &str) -> String {
    let mut words = value.split_whitespace().map(str::to_string).collect::<Vec<_>>();
    let mut redact_next = false;
    for word in &mut words {
        let lower = word.to_ascii_lowercase();
        if redact_next {
            *word = "[REDACTED]".into();
            redact_next = false;
            continue;
        }
        if matches!(lower.as_str(), "bearer" | "authorization:" | "password" | "password:" | "token" | "token:" | "api_key" | "apikey") {
            redact_next = true;
            continue;
        }
        if lower.starts_with("token=") || lower.starts_with("password=") || lower.starts_with("api_key=") || lower.starts_with("apikey=") {
            let key = word.split('=').next().unwrap_or("secret");
            *word = format!("{key}=[REDACTED]");
        }
    }
    words.join(" ")
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}


pub fn execute_action(
    data_dir: &Path,
    port: u16,
    action: AiAction,
) -> Result<AdbResult, String> {
    let settings = settings::load(data_dir)?;
    if !settings.enabled {
        return Err("NekoAI is disabled".into());
    }
    if settings.control_mode == "manual" {
        return Err("AI control mode is manual; Android control actions are disabled".into());
    }
    enforce_package_permission(port, &settings)?;
    enforce_action_rate(settings.max_actions_per_minute)?;

    let result = match action {
        AiAction::Tap { x, y } => adb::input_tap(port, x, y)?,
        AiAction::Hold { x, y, duration_ms } => adb::input_hold(port, x, y, duration_ms)?,
        AiAction::Swipe { x1, y1, x2, y2, duration_ms }
        | AiAction::Drag { x1, y1, x2, y2, duration_ms } => {
            adb::input_swipe(port, x1, y1, x2, y2, duration_ms)?
        }
        AiAction::Key { keycode } => adb::input_keyevent(port, keycode)?,
        AiAction::Text { text } => adb::input_text(port, text)?,
    };

    append_control_log(data_dir, port, result.success)?;
    Ok(result)
}

fn enforce_action_rate(limit: u32) -> Result<(), String> {
    let now = Instant::now();
    let window = Duration::from_secs(60);
    let queue = ACTION_TIMES.get_or_init(|| Mutex::new(VecDeque::new()));
    let mut queue = queue.lock().map_err(|_| "AI action limiter lock poisoned".to_string())?;

    while queue.front().map(|time| now.duration_since(*time) >= window).unwrap_or(false) {
        queue.pop_front();
    }

    if queue.len() >= limit as usize {
        return Err(format!("AI action rate limit reached ({limit} actions/minute)"));
    }

    queue.push_back(now);
    Ok(())
}

fn append_control_log(data_dir: &Path, port: u16, success: bool) -> Result<(), String> {
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
    writeln!(file, "[{timestamp}] android-control port={port} success={success}")
        .map_err(|e| format!("Unable to write AI control log: {e}"))
}


fn enforce_package_permission(port: u16, settings: &AiSettings) -> Result<(), String> {
    if !settings.enforce_package_allowlist {
        return Ok(());
    }
    if settings.allowed_packages.is_empty() {
        return Err("Per-game AI permissions are enabled but the allowlist is empty".into());
    }
    let result = adb::shell(port, "dumpsys window windows | grep -E 'mCurrentFocus|mFocusedApp' | head -n 1".into())?;
    let foreground = result.stdout
        .split_whitespace()
        .find_map(|token| token.split_once('/').map(|(pkg, _)| pkg.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '_').to_string()))
        .filter(|pkg| !pkg.is_empty())
        .ok_or_else(|| "Unable to determine the foreground Android package".to_string())?;
    if settings.allowed_packages.iter().any(|pkg| pkg == &foreground) {
        Ok(())
    } else {
        Err(format!("AI control is not permitted for foreground package {foreground}"))
    }
}

pub fn execute_actions(
    data_dir: &Path,
    port: u16,
    actions: Vec<AiAction>,
) -> Result<Vec<AdbResult>, String> {
    if actions.is_empty() {
        return Err("AI action queue cannot be empty".into());
    }
    if actions.len() > 100 {
        return Err("AI action queue is limited to 100 actions".into());
    }
    let settings = settings::load(data_dir)?;
    if settings.control_mode == "manual" {
        return Err("AI control mode is manual; action queues are disabled".into());
    }
    if settings.control_mode == "assistant" && actions.len() > 5 {
        return Err("Assistant mode limits a queue to 5 actions; use full-automation for longer user-authorized queues".into());
    }

    CANCEL_ACTIONS.store(false, Ordering::SeqCst);
    let mut results = Vec::with_capacity(actions.len());

    for action in actions {
        if CANCEL_ACTIONS.load(Ordering::SeqCst) {
            return Err("AI action queue cancelled".into());
        }

        let result = execute_action(data_dir, port, action)?;
        let failed = !result.success;
        results.push(result);
        if failed {
            break;
        }
    }

    Ok(results)
}

pub fn cancel_actions() {
    CANCEL_ACTIONS.store(true, Ordering::SeqCst);
}
