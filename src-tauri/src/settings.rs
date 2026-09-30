use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub enabled: bool,
    pub mode: String,
    pub endpoint: String,
    pub model: String,
    pub vision_model: String,
    pub detector_model: String,
    #[serde(default = "default_control_mode")]
    pub control_mode: String,
    #[serde(default = "default_helper_mode")]
    pub helper_mode: String,
    #[serde(default)]
    pub enforce_package_allowlist: bool,
    #[serde(default)]
    pub allowed_packages: Vec<String>,
    pub max_actions_per_minute: u32,
    pub max_capture_fps: u32,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "local".into(),
            endpoint: "http://127.0.0.1:11434/v1".into(),
            model: "qwen2.5:3b".into(),
            vision_model: "qwen2.5vl:3b".into(),
            detector_model: "yolov8n.onnx".into(),
            control_mode: "manual".into(),
            helper_mode: "ui".into(),
            enforce_package_allowlist: false,
            allowed_packages: Vec::new(),
            max_actions_per_minute: 60,
            max_capture_fps: 10,
        }
    }
}

fn default_control_mode() -> String { "manual".into() }
fn default_helper_mode() -> String { "ui".into() }

fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("ai-settings.json")
}

pub fn load(data_dir: &Path) -> Result<AiSettings, String> {
    let path = settings_path(data_dir);
    if !path.exists() {
        return Ok(AiSettings::default());
    }

    let bytes = fs::read(&path).map_err(|e| format!("Unable to read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Invalid AI settings: {e}"))
}

pub fn save(data_dir: &Path, settings: AiSettings) -> Result<AiSettings, String> {
    validate(&settings)?;
    fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
    let encoded = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(settings_path(data_dir), encoded).map_err(|e| e.to_string())?;
    Ok(settings)
}

pub fn emergency_stop(data_dir: &Path) -> Result<AiSettings, String> {
    let mut settings = load(data_dir)?;
    settings.enabled = false;
    save(data_dir, settings)
}

fn validate(settings: &AiSettings) -> Result<(), String> {
    if !matches!(settings.mode.as_str(), "local" | "ollama" | "openai-compatible") {
        return Err("AI mode must be local, ollama, or openai-compatible".into());
    }

    if settings.enabled && settings.mode != "local" && settings.endpoint.trim().is_empty() {
        return Err("Remote AI modes require an endpoint".into());
    }

    if settings.model.trim().is_empty() {
        return Err("AI model cannot be empty".into());
    }
    if !matches!(settings.control_mode.as_str(), "manual" | "assistant" | "accessibility" | "full-automation") {
        return Err("AI control mode must be manual, assistant, accessibility, or full-automation".into());
    }
    if !matches!(settings.helper_mode.as_str(), "inventory" | "quest" | "ui" | "repetitive-task") {
        return Err("AI helper mode must be inventory, quest, ui, or repetitive-task".into());
    }
    if settings.allowed_packages.len() > 200 {
        return Err("AI package allowlist is limited to 200 packages".into());
    }

    if !(1..=600).contains(&settings.max_actions_per_minute) {
        return Err("AI action limit must be between 1 and 600 actions/minute".into());
    }

    if !(1..=120).contains(&settings.max_capture_fps) {
        return Err("AI capture FPS must be between 1 and 120".into());
    }

    Ok(())
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub default_android_version: String,
    pub default_profile: String,
    pub default_adb_enabled: bool,
    pub default_headless: bool,
    pub confirm_dangerous_actions: bool,
    pub api_enabled: bool,
    pub api_port: u16,
    pub api_token: String,
    pub first_run_completed: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_android_version: "16".into(),
            default_profile: "Gaming Phone".into(),
            default_adb_enabled: false,
            default_headless: false,
            confirm_dangerous_actions: true,
            api_enabled: false,
            api_port: 37891,
            api_token: String::new(),
            first_run_completed: false,
        }
    }
}

fn app_settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("app-settings.json")
}

pub fn load_app(data_dir: &Path) -> Result<AppSettings, String> {
    let path = app_settings_path(data_dir);
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let bytes = fs::read(&path).map_err(|e| format!("Unable to read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Invalid app settings: {e}"))
}

pub fn save_app(data_dir: &Path, settings: AppSettings) -> Result<AppSettings, String> {
    if !matches!(
        settings.default_android_version.as_str(),
        "9" | "10" | "11" | "12" | "13" | "14" | "15" | "16"
    ) {
        return Err("Default Android version must be between 9 and 16".into());
    }
    if settings.default_profile.trim().is_empty() {
        return Err("Default profile cannot be empty".into());
    }
    if settings.api_port == 0 {
        return Err("Automation API port must be between 1 and 65535".into());
    }
    if settings.api_enabled && settings.api_token.len() < 16 {
        return Err("Automation API token must be at least 16 characters when the API is enabled".into());
    }

    fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
    let encoded = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(app_settings_path(data_dir), encoded).map_err(|e| e.to_string())?;
    Ok(settings)
}
