use crate::media;
use serde::{Deserialize,Serialize};
use std::{
    collections::HashMap,
    env,fs,
    io::{BufRead,BufReader},
    path::{Path,PathBuf},
    process::{Command,Stdio},
    sync::{Mutex,OnceLock},
    thread,
    time::{SystemTime,UNIX_EPOCH},
};

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",default)]
pub struct FfmpegSettings{
    pub custom_ffmpeg_path:String,
    pub custom_ffprobe_path:String,
    pub preferred_hardware_encoder:String,
    pub recording_quality:String,
    pub obs_friendly:bool,
}
impl Default for FfmpegSettings{
    fn default()->Self{Self{custom_ffmpeg_path:String::new(),custom_ffprobe_path:String::new(),preferred_hardware_encoder:"auto".into(),recording_quality:"high".into(),obs_friendly:true}}
}
#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct MediaJobStatus{
    pub id:String,pub input:String,pub output:String,pub status:String,pub progress:f64,pub message:String,pub pid:Option<u32>,
}
static JOBS:OnceLock<Mutex<HashMap<String,MediaJobStatus>>>=OnceLock::new();
fn jobs()->&'static Mutex<HashMap<String,MediaJobStatus>>{JOBS.get_or_init(||Mutex::new(HashMap::new()))}
fn settings_path(data_dir:&Path)->PathBuf{data_dir.join("ffmpeg-settings.json")}
pub fn load_settings(data_dir:&Path)->Result<FfmpegSettings,String>{
    let p=settings_path(data_dir);if !p.exists(){return Ok(FfmpegSettings::default());}
    serde_json::from_slice(&fs::read(p).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
}
pub fn save_settings(data_dir:&Path,s:FfmpegSettings)->Result<FfmpegSettings,String>{
    if !matches!(s.recording_quality.as_str(),"low"|"medium"|"high"|"lossless"){return Err("Invalid recording quality".into());}
    fs::write(settings_path(data_dir),serde_json::to_vec_pretty(&s).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;Ok(s)
}
fn find_in_path(name:&str)->Option<PathBuf>{env::var_os("PATH").and_then(|paths|env::split_paths(&paths).map(|d|d.join(name)).find(|p|p.is_file()))}
fn ffmpeg_path(s:&FfmpegSettings)->Result<PathBuf,String>{
    if !s.custom_ffmpeg_path.trim().is_empty()&&Path::new(&s.custom_ffmpeg_path).is_file(){return Ok(PathBuf::from(&s.custom_ffmpeg_path));}
    find_in_path(if cfg!(windows){"ffmpeg.exe"}else{"ffmpeg"}).ok_or_else(||"FFmpeg not found".to_string())
}
fn ffprobe_path(s:&FfmpegSettings)->Option<PathBuf>{
    if !s.custom_ffprobe_path.trim().is_empty()&&Path::new(&s.custom_ffprobe_path).is_file(){return Some(PathBuf::from(&s.custom_ffprobe_path));}
    find_in_path(if cfg!(windows){"ffprobe.exe"}else{"ffprobe"})
}
fn duration_seconds(input:&str,s:&FfmpegSettings)->Option<f64>{
    let p=ffprobe_path(s)?;
    let out=Command::new(p).args(["-v","error","-show_entries","format=duration","-of","default=noprint_wrappers=1:nokey=1",input]).output().ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}
fn id()->String{format!("media-{}",SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis())}
fn update(id:&str,status:&str,progress:f64,message:&str,pid:Option<u32>){
    if let Ok(mut map)=jobs().lock(){if let Some(j)=map.get_mut(id){j.status=status.into();j.progress=progress.clamp(0.0,100.0);j.message=message.into();j.pid=pid;}}
}
pub fn start(data_dir:PathBuf,request:media::MediaJobRequest)->Result<MediaJobStatus,String>{
    let settings=load_settings(&data_dir)?;
    let exe=ffmpeg_path(&settings)?;
    if !Path::new(&request.input).is_file(){return Err("Input file does not exist".into());}
    let job_id=id();
    let status=MediaJobStatus{id:job_id.clone(),input:request.input.clone(),output:request.output.clone(),status:"queued".into(),progress:0.0,message:"Queued".into(),pid:None};
    jobs().lock().map_err(|_|"media job lock poisoned")?.insert(job_id.clone(),status.clone());
    thread::spawn(move||{
        let duration=duration_seconds(&request.input,&settings).unwrap_or(0.0);
        let mut args=vec!["-hide_banner".to_string(),"-y".into(),"-i".into(),request.input.clone()];
        if request.operation=="remux"{args.extend(["-c".into(),"copy".into()]);}
        else if matches!(request.operation.as_str(),"audio"|"extract-audio"){
            args.extend(["-vn".into(),"-c:a".into(),request.audio_codec.clone()]);
        }else{
            let mut codec=request.video_codec.clone();
            if settings.preferred_hardware_encoder!="auto"{codec=settings.preferred_hardware_encoder.clone();}
            args.extend(["-c:v".into(),codec,"-c:a".into(),request.audio_codec.clone()]);
            let q=match settings.recording_quality.as_str(){"low"=>"30","medium"=>"24","lossless"=>"0",_=>"18"};
            args.extend(["-crf".into(),q.into()]);
            if settings.obs_friendly{args.extend(["-pix_fmt".into(),"yuv420p".into(),"-movflags".into(),"+faststart".into()]);}
            if let (Some(w),Some(h))=(request.width,request.height){if w>0&&h>0{args.extend(["-vf".into(),format!("scale={w}:{h}")]);}}
            if let Some(fps)=request.fps{if fps>0{args.extend(["-r".into(),fps.min(240).to_string()]);}}
        }
        args.extend(["-progress".into(),"pipe:1".into(),"-nostats".into(),request.output.clone()]);
        let mut child=match Command::new(exe).args(&args).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn(){
            Ok(c)=>c,Err(e)=>{update(&job_id,"failed",0.0,&e.to_string(),None);return;}
        };
        let pid=child.id();update(&job_id,"running",1.0,"FFmpeg running",Some(pid));
        if let Some(stdout)=child.stdout.take(){
            let reader=BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok){
                if let Some(v)=line.strip_prefix("out_time_ms="){
                    if let Ok(us)=v.parse::<f64>(){if duration>0.0{update(&job_id,"running",(us/1_000_000.0/duration)*100.0,"Encoding",Some(pid));}}
                }
            }
        }
        match child.wait(){
            Ok(s) if s.success()=>update(&job_id,"completed",100.0,"Completed",None),
            Ok(s)=>update(&job_id,"failed",0.0,&format!("FFmpeg exited {s}"),None),
            Err(e)=>update(&job_id,"failed",0.0,&e.to_string(),None),
        }
    });
    Ok(status)
}
pub fn list()->Result<Vec<MediaJobStatus>,String>{Ok(jobs().lock().map_err(|_|"media job lock poisoned")?.values().cloned().collect())}
pub fn cancel(id:&str)->Result<MediaJobStatus,String>{
    let j=jobs().lock().map_err(|_|"media job lock poisoned")?.get(id).cloned().ok_or("Media job not found")?;
    if let Some(pid)=j.pid{
        if cfg!(windows){let _=Command::new("taskkill").args(["/PID",&pid.to_string(),"/T","/F"]).output();}
        else{let _=Command::new("kill").args(["-TERM",&pid.to_string()]).output();}
    }
    update(id,"cancelled",j.progress,"Cancelled",None);
    jobs().lock().map_err(|_|"media job lock poisoned")?.get(id).cloned().ok_or("Media job not found".into())
}
