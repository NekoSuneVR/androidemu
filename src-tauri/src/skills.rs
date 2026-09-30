use crate::{adb, models::AdbResult};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillControl {
    pub id: String,
    pub label: String,
    pub action: String,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub keycode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillUiRegion {
    pub id: String,
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub packages: Vec<String>,
    pub orientations: Vec<String>,
    pub controls: Vec<SkillControl>,
    pub ui_regions: Vec<SkillUiRegion>,
    pub system_prompt: String,
    pub repository_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiGameState {
    pub package_name: String,
    pub activity: String,
    pub orientation: String,
    pub display_size: String,
}

fn skills_dir(data_dir: &Path) -> PathBuf { data_dir.join("skills") }

pub fn generic_android_skill() -> SkillManifest {
    SkillManifest {
        schema_version: 1,
        id: "generic-android".into(),
        name: "Generic Android".into(),
        version: "1.0.0".into(),
        packages: Vec::new(),
        orientations: vec!["portrait".into(), "landscape".into()],
        controls: vec![
            SkillControl { id: "tap".into(), label: "Tap".into(), action: "tap".into(), x: None, y: None, keycode: None },
            SkillControl { id: "back".into(), label: "Back".into(), action: "key".into(), x: None, y: None, keycode: Some("KEYCODE_BACK".into()) },
            SkillControl { id: "home".into(), label: "Home".into(), action: "key".into(), x: None, y: None, keycode: Some("KEYCODE_HOME".into()) },
        ],
        ui_regions: Vec::new(),
        system_prompt: "Operate generic Android UI only through NekoDroid virtual controls and describe uncertainty before acting.".into(),
        repository_url: None,
    }
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("Skill ID may contain only letters, numbers, '-' and '_'".into());
    }
    Ok(())
}

fn validate(skill: &SkillManifest) -> Result<(), String> {
    validate_id(&skill.id)?;
    if skill.schema_version != 1 { return Err("Unsupported skill schema version".into()); }
    if skill.name.trim().is_empty() || skill.version.trim().is_empty() { return Err("Skill name and version are required".into()); }
    if skill.system_prompt.len() > 32_000 { return Err("Skill prompt is too large".into()); }
    Ok(())
}

pub fn list(data_dir: &Path) -> Result<Vec<SkillManifest>, String> {
    let dir = skills_dir(data_dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut out = vec![generic_android_skill()];
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|v| v.to_str()) != Some("json") { continue; }
        let bytes = fs::read(entry.path()).map_err(|e| e.to_string())?;
        if let Ok(skill) = serde_json::from_slice::<SkillManifest>(&bytes) { out.push(skill); }
    }
    out.sort_by(|a,b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub fn save(data_dir: &Path, skill: SkillManifest) -> Result<SkillManifest, String> {
    validate(&skill)?;
    if skill.id == "generic-android" { return Err("The built-in generic skill cannot be overwritten".into()); }
    let dir = skills_dir(data_dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(format!("{}.json", skill.id)), serde_json::to_vec_pretty(&skill).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(skill)
}

pub fn import_file(data_dir: &Path, source: String) -> Result<SkillManifest, String> {
    let bytes = fs::read(&source).map_err(|e| format!("Unable to read skill file: {e}"))?;
    let skill: SkillManifest = serde_json::from_slice(&bytes).map_err(|e| format!("Invalid skill manifest: {e}"))?;
    save(data_dir, skill)
}

pub fn export_file(data_dir: &Path, id: String, destination: String) -> Result<String, String> {
    validate_id(&id)?;
    let skill = list(data_dir)?.into_iter().find(|s| s.id == id).ok_or_else(|| "Skill not found".to_string())?;
    let dest = PathBuf::from(destination);
    if let Some(parent) = dest.parent() { if !parent.as_os_str().is_empty() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; } }
    fs::write(&dest, serde_json::to_vec_pretty(&skill).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

pub fn game_state(port: u16) -> Result<AiGameState, String> {
    let focus: AdbResult = adb::shell(port, "dumpsys window windows | grep -E 'mCurrentFocus|mFocusedApp' | head -n 1".into())?;
    let token = focus.stdout.split_whitespace().find(|v| v.contains('/')).unwrap_or("");
    let clean = token.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '_' | '$')));
    let (package_name, activity) = clean.split_once('/').map(|(a,b)|(a.to_string(),b.to_string())).unwrap_or_default();

    let orientation_result = adb::shell(port, "dumpsys input | grep -m1 'SurfaceOrientation'".into())?;
    let orientation_code = orientation_result.stdout.split(':').last().unwrap_or("").trim();
    let orientation = match orientation_code { "1" | "3" => "landscape", "0" | "2" => "portrait", _ => "unknown" }.to_string();

    let size_result = adb::shell(port, "wm size".into())?;
    let display_size = size_result.stdout.lines().find_map(|line| line.split_once(':').map(|(_,v)| v.trim().to_string())).unwrap_or_default();

    Ok(AiGameState { package_name, activity, orientation, display_size })
}
