use crate::runtime::{self,RuntimeState};
use memmap2::MmapMut;
use serde::Serialize;
use std::{fs::{self,OpenOptions},io::Write,path::PathBuf,time::{SystemTime,UNIX_EPOCH}};

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct SharedFrameInfo{
    pub path:String,
    pub length:u64,
    pub updated_at:u64,
    pub format:String,
}
pub fn capture(state:&RuntimeState,id:&str)->Result<SharedFrameInfo,String>{
    let dir=state.data_dir.join("shared-frames");fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let temp=dir.join(format!("{id}.capture.ppm"));
    runtime::capture_framebuffer(state,id,temp.to_string_lossy().to_string())?;
    let bytes=fs::read(&temp).map_err(|e|e.to_string())?;
    let target:PathBuf=dir.join(format!("{id}.frame"));
    let file=OpenOptions::new().read(true).write(true).create(true).truncate(true).open(&target).map_err(|e|e.to_string())?;
    file.set_len(bytes.len() as u64).map_err(|e|e.to_string())?;
    let mut map=unsafe{MmapMut::map_mut(&file).map_err(|e|e.to_string())?};
    map[..bytes.len()].copy_from_slice(&bytes);
    map.flush().map_err(|e|e.to_string())?;
    let _=fs::remove_file(temp);
    let updated_at=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    Ok(SharedFrameInfo{path:target.to_string_lossy().to_string(),length:bytes.len() as u64,updated_at,format:"ppm-p6".into()})
}
