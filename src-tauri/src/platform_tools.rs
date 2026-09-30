use crate::adb;
use serde::{Deserialize,Serialize};
use std::{fs,path::{Path,PathBuf}};

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
#[serde(rename_all="camelCase")]
pub struct PlatformToolsSettings{
    pub directory:String,
}
fn path(data_dir:&Path)->PathBuf{data_dir.join("platform-tools.json")}
pub fn load(data_dir:&Path)->Result<PlatformToolsSettings,String>{
    let p=path(data_dir);
    if !p.exists(){return Ok(PlatformToolsSettings::default());}
    serde_json::from_slice(&fs::read(p).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
}
pub fn apply(settings:&PlatformToolsSettings)->Result<(),String>{
    if settings.directory.trim().is_empty(){adb::set_executable_override(None);return Ok(());}
    let dir=PathBuf::from(settings.directory.trim());
    let candidate=dir.join(if cfg!(windows){"adb.exe"}else{"adb"});
    if !candidate.is_file(){return Err(format!("ADB was not found at {}",candidate.display()));}
    adb::set_executable_override(Some(candidate));Ok(())
}
pub fn save(data_dir:&Path,s:PlatformToolsSettings)->Result<PlatformToolsSettings,String>{
    apply(&s)?;
    fs::write(path(data_dir),serde_json::to_vec_pretty(&s).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(s)
}
