use crate::{adb, models::AdbResult};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSettings {
    pub package_name: String,
    #[serde(default)]
    pub renderer: Option<String>,
    #[serde(default)]
    pub android_version: Option<String>,
    #[serde(default)]
    pub orientation: Option<String>,
    #[serde(default)]
    pub dpi: Option<u32>,
    #[serde(default)]
    pub fps: Option<u32>,
    #[serde(default)]
    pub keymap_id: Option<String>,
    #[serde(default)]
    pub ai_skill_id: Option<String>,
    #[serde(default)]
    pub compatibility_rating: Option<String>,
    #[serde(default)]
    pub known_issues: Vec<String>,
    #[serde(default)]
    pub crash_diagnostics: bool,
    #[serde(default)]
    pub notes: String,
}

fn dir(data_dir:&Path)->PathBuf{data_dir.join("game-settings")}
fn safe_name(pkg:&str)->Result<String,String>{
    if pkg.is_empty() || !pkg.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c,'.'|'_'|'-')){return Err("Invalid package name".into());}
    Ok(pkg.replace('.',"_"))
}
pub fn list(data_dir:&Path)->Result<Vec<GameSettings>,String>{
    let d=dir(data_dir);fs::create_dir_all(&d).map_err(|e|e.to_string())?;
    let mut out=Vec::new();
    for e in fs::read_dir(d).map_err(|e|e.to_string())?{
        let e=e.map_err(|e|e.to_string())?;
        if e.path().extension().and_then(|v|v.to_str())!=Some("json"){continue;}
        let b=fs::read(e.path()).map_err(|e|e.to_string())?;
        if let Ok(v)=serde_json::from_slice::<GameSettings>(&b){out.push(v);}
    }
    out.sort_by(|a,b|a.package_name.cmp(&b.package_name));Ok(out)
}
pub fn save(data_dir:&Path,settings:GameSettings)->Result<GameSettings,String>{
    let name=safe_name(&settings.package_name)?;
    if let Some(r)=settings.renderer.as_deref(){if !matches!(r,"auto"|"vulkan"|"opengl"|"directx"|"software"){return Err("Unsupported renderer".into());}}
    if let Some(v)=settings.android_version.as_deref(){if !matches!(v,"9"|"10"|"11"|"12"|"12L"|"13"|"14"|"15"|"16"){return Err("Unsupported Android version".into());}}
    if let Some(o)=settings.orientation.as_deref(){if !matches!(o,"portrait"|"landscape"|"reverse-portrait"|"reverse-landscape"|"automatic"){return Err("Unsupported orientation".into());}}
    if let Some(r)=settings.compatibility_rating.as_deref(){if !matches!(r,"unknown"|"good"|"partial"|"broken"){return Err("Compatibility rating must be unknown, good, partial, or broken".into());}}
    if let Some(dpi)=settings.dpi{if !(72..=1000).contains(&dpi){return Err("DPI must be 72..1000".into());}}
    if let Some(fps)=settings.fps{if !matches!(fps,30|60|90|120|144|165|240){return Err("Unsupported FPS".into());}}
    let d=dir(data_dir);fs::create_dir_all(&d).map_err(|e|e.to_string())?;
    fs::write(d.join(format!("{name}.json")),serde_json::to_vec_pretty(&settings).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(settings)
}
pub fn remove(data_dir:&Path,package_name:String)->Result<(),String>{
    let p=dir(data_dir).join(format!("{}.json",safe_name(&package_name)?));
    if p.exists(){fs::remove_file(p).map_err(|e|e.to_string())?;}Ok(())
}
pub fn apply(port:u16,settings:&GameSettings)->Result<Vec<AdbResult>,String>{
    let mut results=Vec::new();
    if let Some(o)=settings.orientation.clone(){results.push(adb::set_orientation(port,o)?);}
    if let Some(dpi)=settings.dpi{results.push(adb::shell(port,format!("wm density {dpi}"))?);}
    if settings.fps.is_some(){results.push(adb::set_refresh_rate(port,settings.fps)?);}
    Ok(results)
}
