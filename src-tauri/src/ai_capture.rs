use crate::adb;
use serde::Serialize;
use sha2::{Digest,Sha256};
use std::{
    env,fs,
    path::{Path,PathBuf},
    process::Command,
    sync::{Mutex,OnceLock},
    time::{Duration,Instant,SystemTime,UNIX_EPOCH},
};

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct AiCaptureResult{
    pub path:String,
    pub changed:bool,
    pub sha256:String,
    pub captured_at:u64,
}
static LAST_CAPTURE:OnceLock<Mutex<Option<(Instant,String)>>>=OnceLock::new();
fn state()->&'static Mutex<Option<(Instant,String)>>{LAST_CAPTURE.get_or_init(||Mutex::new(None))}
fn find(names:&[&str])->Option<PathBuf>{
    env::var_os("PATH").and_then(|paths|names.iter().find_map(|name|env::split_paths(&paths).map(|d|d.join(name)).find(|p|p.is_file())))
}
fn hash_file(path:&Path)->Result<String,String>{
    let bytes=fs::read(path).map_err(|e|e.to_string())?;
    Ok(format!("{:x}",Sha256::digest(bytes)))
}
pub fn capture(port:u16,destination:String,max_fps:u32)->Result<AiCaptureResult,String>{
    let max_fps=max_fps.clamp(1,60);
    {
        let guard=state().lock().map_err(|_|"capture lock poisoned")?;
        if let Some((at,_))=&*guard{
            let wait=Duration::from_secs_f64(1.0/max_fps as f64);
            if at.elapsed()<wait{return Err("AI capture frame-rate limiter is active".into());}
        }
    }
    let result=adb::screenshot(port,destination.clone())?;
    if !result.success{return Err(result.stderr);}
    let hash=hash_file(Path::new(&destination))?;
    let mut guard=state().lock().map_err(|_|"capture lock poisoned")?;
    let changed=guard.as_ref().map(|(_,old)|old!=&hash).unwrap_or(true);
    *guard=Some((Instant::now(),hash.clone()));
    let captured_at=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    Ok(AiCaptureResult{path:destination,changed,sha256:hash,captured_at})
}
pub fn capture_roi(port:u16,destination:String,x:u32,y:u32,width:u32,height:u32,max_fps:u32)->Result<AiCaptureResult,String>{
    if width==0||height==0||width>8192||height>8192{return Err("Invalid ROI dimensions".into());}
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    let temp=env::temp_dir().join(format!("nekodroid-ai-{stamp}.png"));
    let base=capture(port,temp.to_string_lossy().to_string(),max_fps)?;
    let ffmpeg=find(if cfg!(windows){&["ffmpeg.exe","ffmpeg"]}else{&["ffmpeg"]}).ok_or("FFmpeg is required for ROI capture")?;
    let out=Command::new(ffmpeg).args(["-hide_banner","-loglevel","error","-y","-i",base.path.as_str(),"-vf",&format!("crop={width}:{height}:{x}:{y}"),&destination]).output().map_err(|e|e.to_string())?;
    let _=fs::remove_file(temp);
    if !out.status.success(){return Err(String::from_utf8_lossy(&out.stderr).to_string());}
    let hash=hash_file(Path::new(&destination))?;
    Ok(AiCaptureResult{path:destination,changed:true,sha256:hash,captured_at:base.captured_at})
}
pub fn ocr(image_path:String)->Result<String,String>{
    let exe=find(if cfg!(windows){&["tesseract.exe","tesseract"]}else{&["tesseract"]}).ok_or("Tesseract OCR not found in PATH")?;
    let out=Command::new(exe).args([image_path.as_str(),"stdout"]).output().map_err(|e|e.to_string())?;
    if !out.status.success(){return Err(String::from_utf8_lossy(&out.stderr).to_string());}
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
pub fn speech_to_text(audio_path:String)->Result<String,String>{
    let exe=find(if cfg!(windows){&["whisper-cli.exe","whisper.exe","whisper-cli"]}else{&["whisper-cli","whisper"]}).ok_or("whisper-cli not found in PATH")?;
    let out=Command::new(exe).arg(audio_path).output().map_err(|e|e.to_string())?;
    if !out.status.success(){return Err(String::from_utf8_lossy(&out.stderr).to_string());}
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
pub fn tts(text:String)->Result<String,String>{
    if text.trim().is_empty(){return Err("TTS text cannot be empty".into());}
    if cfg!(windows){
        let escaped=text.replace('\'',"''");
        let script=format!("Add-Type -AssemblyName System.Speech; $s=New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.Speak('{escaped}')");
        let out=Command::new("powershell").args(["-NoProfile","-Command",&script]).output().map_err(|e|e.to_string())?;
        if !out.status.success(){return Err(String::from_utf8_lossy(&out.stderr).to_string());}
        return Ok("Spoken with Windows SAPI".into());
    }
    let exe=find(&["espeak-ng","espeak"]).ok_or("espeak/espeak-ng not found in PATH")?;
    let out=Command::new(exe).arg(text).output().map_err(|e|e.to_string())?;
    if !out.status.success(){return Err(String::from_utf8_lossy(&out.stderr).to_string());}
    Ok("Spoken with host TTS".into())
}
pub fn input_visualizer(port:u16)->Result<String,String>{
    let result=adb::shell(port,"getevent -lp 2>/dev/null | head -n 240".into())?;
    if result.success{Ok(result.stdout)}else{Err(result.stderr)}
}
