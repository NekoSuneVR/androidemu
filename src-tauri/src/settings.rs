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
            max_actions_per_minute: 60,
            max_capture_fps: 10,
        }
    }
}

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

    if !(1..=600).contains(&settings.max_actions_per_minute) {
        return Err("AI action limit must be between 1 and 600 actions/minute".into());
    }

    if !(1..=120).contains(&settings.max_capture_fps) {
        return Err("AI capture FPS must be between 1 and 120".into());
    }

    Ok(())
}
