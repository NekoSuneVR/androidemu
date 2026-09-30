use crate::{adb,media::MediaResult};
use std::{env,path::Path,process::Command};

fn ffmpeg()->Result<String,String>{
    let name=if cfg!(windows){"ffmpeg.exe"}else{"ffmpeg"};
    let paths=env::var_os("PATH").ok_or("PATH unavailable")?;
    env::split_paths(&paths).map(|d|d.join(name)).find(|p|p.is_file()).map(|p|p.to_string_lossy().to_string()).ok_or("FFmpeg not found")
}

pub fn android_audio_capture(port:u16,destination:String,seconds:u32)->Result<MediaResult,String>{
    if destination.trim().is_empty(){return Err("Destination is required".into());}
    let seconds=seconds.clamp(1,300);
    let remote="/sdcard/Download/nekodroid-audio.wav";
    let command=format!("if command -v tinycap >/dev/null 2>&1; then timeout {seconds} tinycap {remote}; elif command -v tinyrecord >/dev/null 2>&1; then timeout {seconds} tinyrecord {remote}; else echo 'No compatible tinycap/tinyrecord capture utility found' >&2; exit 127; fi");
    let capture=adb::root_shell(port,command)?;
    if !capture.success{return Ok(MediaResult{success:false,exit_code:capture.exit_code,stdout:capture.stdout,stderr:capture.stderr,output_path:destination});}
    let pulled=adb::pull(port,remote.into(),destination.clone())?;
    let _=adb::shell(port,format!("rm -f {remote}"));
    Ok(MediaResult{success:pulled.success,exit_code:pulled.exit_code,stdout:pulled.stdout,stderr:pulled.stderr,output_path:destination})
}

pub fn mix_microphone(input:String,output:String,microphone:String)->Result<MediaResult,String>{
    if !Path::new(&input).is_file(){return Err("Input file does not exist".into());}
    if microphone.trim().is_empty(){return Err("Microphone device is required".into());}
    let exe=ffmpeg()?;
    let mut args=vec!["-hide_banner".to_string(),"-y".into(),"-i".into(),input.clone()];
    if cfg!(windows){args.extend(["-f".into(),"dshow".into(),"-i".into(),format!("audio={microphone}")]);}
    else{args.extend(["-f".into(),"pulse".into(),"-i".into(),microphone.clone()]);}
    args.extend(["-filter_complex".into(),"[0:a][1:a]amix=inputs=2:duration=first:dropout_transition=2[a]".into(),"-map".into(),"0:v?".into(),"-map".into(),"[a]".into(),"-c:v".into(),"copy".into(),"-c:a".into(),"aac".into(),output.clone()]);
    let out=Command::new(exe).args(&args).output().map_err(|e|e.to_string())?;
    Ok(MediaResult{success:out.status.success(),exit_code:out.status.code(),stdout:String::from_utf8_lossy(&out.stdout).trim().to_string(),stderr:String::from_utf8_lossy(&out.stderr).trim().to_string(),output_path:output})
}

pub fn virtual_camera(input:String,device:String)->Result<MediaResult,String>{
    if !Path::new(&input).is_file(){return Err("Input file does not exist".into());}
    if cfg!(windows){return Err("Direct FFmpeg virtual-camera output requires a Windows virtual-camera driver/provider. Configure OBS Virtual Camera or another provider and use NekoDroid's OBS-friendly output.".into());}
    if !device.starts_with("/dev/video"){return Err("Linux virtual camera device must be /dev/videoN".into());}
    let exe=ffmpeg()?;
    let out=Command::new(exe).args(["-hide_banner","-re","-i",&input,"-vf","format=yuv420p","-f","v4l2",&device]).output().map_err(|e|e.to_string())?;
    Ok(MediaResult{success:out.status.success(),exit_code:out.status.code(),stdout:String::from_utf8_lossy(&out.stdout).trim().to_string(),stderr:String::from_utf8_lossy(&out.stderr).trim().to_string(),output_path:device})
}
