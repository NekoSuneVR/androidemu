use crate::{adb, display, models::AdbResult};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyBinding {
    pub input: String,
    pub action: String,
    #[serde(default)]
    pub x: Option<i32>,
    #[serde(default)]
    pub y: Option<i32>,
    #[serde(default)]
    pub x2: Option<i32>,
    #[serde(default)]
    pub y2: Option<i32>,
    #[serde(default)]
    pub duration_ms: Option<u32>,
    #[serde(default)]
    pub keycode: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeymapProfile {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub package_name: String,
    #[serde(default)]
    pub overlay_visible: bool,
    #[serde(default)]
    pub bindings: Vec<KeyBinding>,
}

fn dir(data_dir: &Path) -> PathBuf { data_dir.join("keymaps") }

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

fn validate(profile: &KeymapProfile) -> Result<(), String> {
    if profile.schema_version != 1 { return Err("Unsupported keymap schema version".into()); }
    if !valid_id(&profile.id) { return Err("Invalid keymap id".into()); }
    if profile.name.trim().is_empty() { return Err("Keymap name cannot be empty".into()); }
    if profile.bindings.len() > 256 { return Err("Keymap is limited to 256 bindings".into()); }
    for binding in &profile.bindings {
        if binding.input.trim().is_empty() { return Err("Keymap input cannot be empty".into()); }
        if !matches!(binding.action.as_str(), "tap" | "hold" | "swipe" | "key" | "text" | "mouse-left" | "mouse-right" | "scroll-up" | "scroll-down" | "joystick-up" | "joystick-down" | "joystick-left" | "joystick-right") {
            return Err(format!("Unsupported keymap action: {}", binding.action));
        }
    }
    Ok(())
}

pub fn list(data_dir: &Path) -> Result<Vec<KeymapProfile>, String> {
    let folder=dir(data_dir);
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let mut out=Vec::new();
    for entry in fs::read_dir(folder).map_err(|e| e.to_string())? {
        let entry=entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|v| v.to_str()) != Some("json") { continue; }
        let bytes=fs::read(entry.path()).map_err(|e| e.to_string())?;
        if let Ok(profile)=serde_json::from_slice::<KeymapProfile>(&bytes) { out.push(profile); }
    }
    out.sort_by(|a,b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub fn save(data_dir: &Path, profile: KeymapProfile) -> Result<KeymapProfile,String> {
    validate(&profile)?;
    let folder=dir(data_dir); fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
    fs::write(folder.join(format!("{}.json",profile.id)),serde_json::to_vec_pretty(&profile).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(profile)
}

pub fn remove(data_dir:&Path,id:&str)->Result<(),String>{
    if !valid_id(id){return Err("Invalid keymap id".into());}
    let path=dir(data_dir).join(format!("{id}.json"));
    if path.exists(){fs::remove_file(path).map_err(|e|e.to_string())?;}
    Ok(())
}

pub fn import_file(data_dir:&Path,source:String)->Result<KeymapProfile,String>{
    let bytes=fs::read(source).map_err(|e|e.to_string())?;
    let profile:KeymapProfile=serde_json::from_slice(&bytes).map_err(|e|format!("Invalid keymap: {e}"))?;
    save(data_dir,profile)
}

pub fn export_file(data_dir:&Path,id:String,destination:String)->Result<String,String>{
    let profile=list(data_dir)?.into_iter().find(|p|p.id==id).ok_or_else(||"Keymap not found".to_string())?;
    let dest=PathBuf::from(destination);
    if let Some(parent)=dest.parent(){if !parent.as_os_str().is_empty(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}}
    fs::write(&dest,serde_json::to_vec_pretty(&profile).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

pub fn execute(port:u16,binding:KeyBinding)->Result<AdbResult,String>{
    match binding.action.as_str(){
        "tap"|"mouse-left"=>{let (x,y)=display::transform(port,binding.x.ok_or("x required")?,binding.y.ok_or("y required")?)?;adb::input_tap(port,x,y)},
        "mouse-right"=>{let (x,y)=display::transform(port,binding.x.ok_or("x required")?,binding.y.ok_or("y required")?)?;adb::input_hold(port,x,y,binding.duration_ms.unwrap_or(500))},
        "hold"=>{let (x,y)=display::transform(port,binding.x.ok_or("x required")?,binding.y.ok_or("y required")?)?;adb::input_hold(port,x,y,binding.duration_ms.unwrap_or(500))},
        "swipe"=>{let (x1,y1,x2,y2)=display::transform_swipe(port,binding.x.ok_or("x required")?,binding.y.ok_or("y required")?,binding.x2.ok_or("x2 required")?,binding.y2.ok_or("y2 required")?)?;adb::input_swipe(port,x1,y1,x2,y2,binding.duration_ms.unwrap_or(300))},
        "key"=>adb::input_keyevent(port,binding.keycode.ok_or("keycode required")?),
        "text"=>adb::input_text(port,binding.text.ok_or("text required")?),
        "scroll-up"=>adb::input_swipe(port,binding.x.unwrap_or(540),binding.y.unwrap_or(1500),binding.x2.unwrap_or(540),binding.y2.unwrap_or(700),binding.duration_ms.unwrap_or(250)),
        "scroll-down"=>adb::input_swipe(port,binding.x.unwrap_or(540),binding.y.unwrap_or(700),binding.x2.unwrap_or(540),binding.y2.unwrap_or(1500),binding.duration_ms.unwrap_or(250)),
        "joystick-up"|"joystick-down"|"joystick-left"|"joystick-right"=>{
            let cx=binding.x.unwrap_or(220); let cy=binding.y.unwrap_or(1800); let d=180;
            let (tx,ty)=match binding.action.as_str(){"joystick-up"=>(cx,cy-d),"joystick-down"=>(cx,cy+d),"joystick-left"=>(cx-d,cy),_=>(cx+d,cy)};
            adb::input_swipe(port,cx,cy,tx,ty,binding.duration_ms.unwrap_or(500))
        },
        _=>Err("Unsupported keymap action".into())
    }
}
