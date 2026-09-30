use serde::{Deserialize,Serialize};
use std::{fs,path::{Path,PathBuf}};

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct PluginManifest{
    pub schema_version:u32,
    pub id:String,
    pub name:String,
    pub version:String,
    pub plugin_type:String,
    #[serde(default)] pub entry:String,
    #[serde(default)] pub capabilities:Vec<String>,
    #[serde(default)] pub source_url:Option<String>,
}
fn dir(data_dir:&Path)->PathBuf{data_dir.join("plugins")}
fn valid_id(id:&str)->bool{!id.is_empty()&&id.chars().all(|c|c.is_ascii_alphanumeric()||matches!(c,'-'|'_'))}
fn validate(p:&PluginManifest)->Result<(),String>{
    if p.schema_version!=1{return Err("Unsupported plugin schema".into());}
    if !valid_id(&p.id){return Err("Invalid plugin id".into());}
    if !matches!(p.plugin_type.as_str(),"renderer"|"ai"|"device-profile"|"game-profile"|"ai-skill"){return Err("Unsupported plugin type".into());}
    Ok(())
}
pub fn list(data_dir:&Path)->Result<Vec<PluginManifest>,String>{
    let d=dir(data_dir);fs::create_dir_all(&d).map_err(|e|e.to_string())?;
    let mut out=Vec::new();
    for e in fs::read_dir(d).map_err(|e|e.to_string())?{
        let e=e.map_err(|e|e.to_string())?;
        if e.path().extension().and_then(|v|v.to_str())!=Some("json"){continue;}
        if let Ok(p)=serde_json::from_slice::<PluginManifest>(&fs::read(e.path()).map_err(|e|e.to_string())?){out.push(p);}
    }
    out.sort_by(|a,b|a.name.to_lowercase().cmp(&b.name.to_lowercase()));Ok(out)
}
pub fn save(data_dir:&Path,p:PluginManifest)->Result<PluginManifest,String>{
    validate(&p)?;let d=dir(data_dir);fs::create_dir_all(&d).map_err(|e|e.to_string())?;
    fs::write(d.join(format!("{}.json",p.id)),serde_json::to_vec_pretty(&p).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;Ok(p)
}
pub fn import_file(data_dir:&Path,path:String)->Result<PluginManifest,String>{
    let p:PluginManifest=serde_json::from_slice(&fs::read(path).map_err(|e|e.to_string())?).map_err(|e|format!("Invalid plugin manifest: {e}"))?;save(data_dir,p)
}
pub fn remove(data_dir:&Path,id:String)->Result<(),String>{
    if !valid_id(&id){return Err("Invalid plugin id".into());}
    let p=dir(data_dir).join(format!("{id}.json"));if p.exists(){fs::remove_file(p).map_err(|e|e.to_string())?;}Ok(())
}
