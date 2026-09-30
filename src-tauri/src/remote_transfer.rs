use crate::adb;
use base64::Engine as _;
use std::{env,fs,path::PathBuf,time::{SystemTime,UNIX_EPOCH}};

pub fn receive_and_push(port:u16,name:String,data_base64:String,destination:Option<String>)->Result<String,String>{
    if name.is_empty()||name.len()>240{return Err("Invalid remote filename".into());}
    let safe=name.chars().map(|c|if c.is_ascii_alphanumeric()||matches!(c,'.'|'-'|'_'){c}else{'_'}).collect::<String>();
    let bytes=base64::engine::general_purpose::STANDARD.decode(data_base64).map_err(|e|format!("Invalid remote file encoding: {e}"))?;
    if bytes.len()>64*1024*1024{return Err("Remote file is limited to 64 MiB per transfer".into());}
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    let path:PathBuf=env::temp_dir().join(format!("nekodroid-remote-{stamp}-{safe}"));
    fs::write(&path,&bytes).map_err(|e|e.to_string())?;
    let target=destination.unwrap_or_else(||"/sdcard/Download/".into());
    let result=adb::push(port,path.to_string_lossy().to_string(),target.clone())?;
    let _=fs::remove_file(&path);
    if !result.success{return Err(result.stderr);}
    Ok(format!("Received {} bytes and pushed {safe} to {target}",bytes.len()))
}
