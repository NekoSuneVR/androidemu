use serde::{Deserialize,Serialize};
use std::{fs,path::{Path,PathBuf},time::Duration};

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CloudAndroidNode{
    pub id:String,
    pub name:String,
    pub base_url:String,
    #[serde(default)] pub token:String,
    #[serde(default)] pub region:String,
    #[serde(default)] pub enabled:bool,
}
fn path(data_dir:&Path)->PathBuf{data_dir.join("cloud-android-nodes.json")}
pub fn list(data_dir:&Path)->Result<Vec<CloudAndroidNode>,String>{
    let p=path(data_dir);if !p.exists(){return Ok(Vec::new());}
    serde_json::from_slice(&fs::read(p).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
}
pub fn save(data_dir:&Path,nodes:Vec<CloudAndroidNode>)->Result<Vec<CloudAndroidNode>,String>{
    for n in &nodes{
        if n.id.trim().is_empty()||n.name.trim().is_empty(){return Err("Cloud node id/name required".into());}
        if !(n.base_url.starts_with("https://")||n.base_url.starts_with("http://127.0.0.1")||n.base_url.starts_with("http://localhost")){return Err(format!("Cloud node {} must use HTTPS (localhost HTTP allowed)",n.name));}
    }
    fs::write(path(data_dir),serde_json::to_vec_pretty(&nodes).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;Ok(nodes)
}
pub fn health(node:CloudAndroidNode)->Result<serde_json::Value,String>{
    let client=reqwest::blocking::Client::builder().timeout(Duration::from_secs(10)).build().map_err(|e|e.to_string())?;
    let mut request=client.get(format!("{}/health",node.base_url.trim_end_matches('/')));
    if !node.token.is_empty(){request=request.bearer_auth(node.token);}
    let response=request.send().map_err(|e|e.to_string())?;
    let status=response.status();
    let body=response.text().unwrap_or_default();
    Ok(serde_json::json!({"id":node.id,"name":node.name,"region":node.region,"ok":status.is_success(),"status":status.as_u16(),"body":body.chars().take(1000).collect::<String>()}))
}
