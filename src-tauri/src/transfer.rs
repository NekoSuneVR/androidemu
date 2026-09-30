use crate::adb;
use serde::Serialize;
use std::{collections::HashMap, process::Command, sync::{Mutex,OnceLock}, thread, time::{SystemTime,UNIX_EPOCH}};

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct TransferJob {
    pub id:String,
    pub direction:String,
    pub source:String,
    pub destination:String,
    pub port:u16,
    pub status:String,
    pub progress:u8,
    pub message:String,
    pub pid:Option<u32>,
}
static JOBS:OnceLock<Mutex<HashMap<String,TransferJob>>>=OnceLock::new();
fn jobs()->&'static Mutex<HashMap<String,TransferJob>>{JOBS.get_or_init(||Mutex::new(HashMap::new()))}
fn new_id()->String{format!("transfer-{}",SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis())}

pub fn start(port:u16,direction:String,source:String,destination:String)->Result<TransferJob,String>{
    if !matches!(direction.as_str(),"push"|"pull"){return Err("direction must be push or pull".into());}
    let id=new_id();
    let job=TransferJob{id:id.clone(),direction:direction.clone(),source:source.clone(),destination:destination.clone(),port,status:"queued".into(),progress:0,message:String::new(),pid:None};
    jobs().lock().map_err(|_|"transfer lock poisoned")?.insert(id.clone(),job.clone());
    thread::spawn(move||{
        let adb_path=match adb::detect_adb().executable{Some(v)=>v,None=>{update(&id,"failed",0,"ADB not found",None);return;}};
        let serial=format!("127.0.0.1:{port}");
        let mut cmd=Command::new(adb_path);
        cmd.args(["-s",&serial,&direction,&source,&destination]);
        match cmd.spawn(){
            Ok(mut child)=>{
                let pid=child.id(); update(&id,"running",10,"Transfer started",Some(pid));
                match child.wait(){
                    Ok(status) if status.success()=>update(&id,"completed",100,"Transfer completed",None),
                    Ok(status)=>update(&id,"failed",0,&format!("ADB transfer exited with {status}"),None),
                    Err(e)=>update(&id,"failed",0,&format!("Transfer failed: {e}"),None),
                }
            }
            Err(e)=>update(&id,"failed",0,&format!("Unable to start transfer: {e}"),None)
        }
    });
    Ok(job)
}
fn update(id:&str,status:&str,progress:u8,message:&str,pid:Option<u32>){
    if let Ok(mut map)=jobs().lock(){if let Some(job)=map.get_mut(id){job.status=status.into();job.progress=progress;job.message=message.into();job.pid=pid;}}
}
pub fn get(id:&str)->Result<TransferJob,String>{jobs().lock().map_err(|_|"transfer lock poisoned")?.get(id).cloned().ok_or_else(||"Transfer job not found".into())}
pub fn list()->Result<Vec<TransferJob>,String>{Ok(jobs().lock().map_err(|_|"transfer lock poisoned")?.values().cloned().collect())}
pub fn cancel(id:&str)->Result<TransferJob,String>{
    let job=get(id)?;
    if let Some(pid)=job.pid{
        if cfg!(windows){let _=Command::new("taskkill").args(["/PID",&pid.to_string(),"/T","/F"]).output();}
        else{let _=Command::new("kill").args(["-TERM",&pid.to_string()]).output();}
    }
    update(id,"cancelled",job.progress,"Transfer cancelled",None);get(id)
}
pub fn retry(id:&str)->Result<TransferJob,String>{let j=get(id)?;start(j.port,j.direction,j.source,j.destination)}
pub fn sync_shared(port:u16,host_path:String)->Result<TransferJob,String>{start(port,"push".into(),host_path,"/sdcard/NekoDroidShared/".into())}
