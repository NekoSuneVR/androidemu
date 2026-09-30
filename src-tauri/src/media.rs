use crate::runtime;
use serde::Serialize;
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCodecCapability {
    pub codec: String,
    pub decode: bool,
    pub encode: bool,
    pub preferred_decoder: Option<String>,
    pub preferred_encoder: Option<String>,
    pub software_fallback: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCodecCapabilityReport {
    pub ffmpeg_found: bool,
    pub capabilities: Vec<MediaCodecCapability>,
    pub hardware_families: Vec<String>,
}

pub fn media_codec_report() -> MediaCodecCapabilityReport {
    let info = detect_ffmpeg();
    let enc = info.executable.as_deref().map(Path::new).map(|p| command_text(p, "-encoders")).unwrap_or_default();
    let dec = info.executable.as_deref().map(Path::new).map(|p| command_text(p, "-decoders")).unwrap_or_default();
    let definitions: &[(&str, &[&str], &[&str])] = &[
        ("h264", &["h264","h264_cuvid","h264_qsv"], &["libx264","h264_nvenc","h264_qsv","h264_amf","h264_vaapi"]),
        ("hevc", &["hevc","hevc_cuvid","hevc_qsv"], &["libx265","hevc_nvenc","hevc_qsv","hevc_amf","hevc_vaapi"]),
        ("vp8", &["vp8","libvpx"], &["libvpx","vp8_vaapi"]),
        ("vp9", &["vp9","libvpx-vp9"], &["libvpx-vp9","vp9_qsv","vp9_vaapi"]),
        ("av1", &["av1","dav1d","libdav1d"], &["libaom-av1","libsvtav1","av1_nvenc","av1_qsv","av1_amf"]),
        ("aac", &["aac"], &["aac"]),
        ("mp3", &["mp3"], &["libmp3lame","mp3"]),
        ("opus", &["opus","libopus"], &["libopus","opus"]),
        ("flac", &["flac"], &["flac"]),
    ];
    let capabilities = definitions.iter().map(|(codec,decs,encs)| {
        let preferred_decoder = decs.iter().find(|name| dec.contains(**name)).map(|s|(*s).to_string());
        let preferred_encoder = encs.iter().find(|name| enc.contains(**name)).map(|s|(*s).to_string());
        MediaCodecCapability {
            codec:(*codec).into(),
            decode:preferred_decoder.is_some(),
            encode:preferred_encoder.is_some(),
            preferred_decoder,
            preferred_encoder,
            software_fallback: match *codec {
                "h264" => dec.contains("h264") && enc.contains("libx264"),
                "hevc" => dec.contains("hevc") && enc.contains("libx265"),
                "vp8" => dec.contains("vp8") && enc.contains("libvpx"),
                "vp9" => dec.contains("vp9") && enc.contains("libvpx-vp9"),
                "av1" => dec.contains("av1") && (enc.contains("libaom-av1") || enc.contains("libsvtav1")),
                _ => true,
            },
        }
    }).collect();

    let mut hardware_families=Vec::new();
    let all=format!("{} {}",info.hardware_encoders.join(" "),info.hardware_decoders.join(" ")).to_ascii_lowercase();
    if all.contains("nvenc") || all.contains("cuvid"){hardware_families.push("NVIDIA NVENC/NVDEC".into());}
    if all.contains("_qsv"){hardware_families.push("Intel Quick Sync".into());}
    if all.contains("_amf"){hardware_families.push("AMD AMF/VCN".into());}
    if all.contains("vaapi"){hardware_families.push("VA-API".into());}

    MediaCodecCapabilityReport { ffmpeg_found:info.found, capabilities, hardware_families }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegInfo {
    pub found: bool,
    pub executable: Option<String>,
    pub version: Option<String>,
    pub hwaccels: Vec<String>,
    pub hardware_encoders: Vec<String>,
    pub hardware_decoders: Vec<String>,
    pub codec_capabilities: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaJobRequest {
    pub input: String,
    pub output: String,
    pub operation: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<u32>,
    pub hardware_decode: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub output_path: String,
}

pub fn detect_ffmpeg() -> FfmpegInfo {
    let candidates: &[&str] = if cfg!(windows) {
        &["ffmpeg.exe", "ffmpeg"]
    } else {
        &["ffmpeg"]
    };

    let executable = if cfg!(windows) {
        runtime::find_runtime_tool("ffmpeg")
    } else {
        candidates.iter().find_map(|name| find_in_path(name))
    };
    let Some(path) = executable else {
        return FfmpegInfo {
            found: false,
            executable: None,
            version: None,
            hwaccels: Vec::new(),
            hardware_encoders: Vec::new(),
            hardware_decoders: Vec::new(),
            codec_capabilities: Vec::new(),
        };
    };

    let version = {
        let mut command = Command::new(&path);
        runtime::apply_runtime_environment(&mut command, &path);
        command.arg("-version").output()
    }
        .ok()
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(str::to_string)
        });

    let hwaccels = {
        let mut command = Command::new(&path);
        runtime::apply_runtime_environment(&mut command, &path);
        command.args(["-hide_banner", "-hwaccels"]).output()
    }
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && *line != "Hardware acceleration methods:")
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let hardware_encoders = detect_codec_names(&path, "-encoders", &["nvenc", "_qsv", "_amf", "vaapi", "videotoolbox"]);
    let hardware_decoders = detect_codec_names(&path, "-decoders", &["cuvid", "_qsv", "vaapi", "videotoolbox", "v4l2m2m"]);
    let encoder_text = command_text(&path, "-encoders");
    let decoder_text = command_text(&path, "-decoders");
    let codec_capabilities = detect_common_codecs(&encoder_text, &decoder_text);

    FfmpegInfo {
        found: true,
        executable: Some(path.to_string_lossy().to_string()),
        version,
        hwaccels,
        hardware_encoders,
        hardware_decoders,
        codec_capabilities,
    }
}

pub fn run_media_job(
    input: String,
    output: String,
    operation: String,
    video_codec: String,
    audio_codec: String,
    width: Option<u32>,
    height: Option<u32>,
    fps: Option<u32>,
    hardware_decode: bool,
) -> Result<MediaResult, String> {
    let input_path = Path::new(&input);
    if !input_path.is_file() {
        return Err(format!("Input file does not exist: {input}"));
    }
    if output.trim().is_empty() {
        return Err("Output path cannot be empty".into());
    }

    if let Some(parent) = Path::new(&output).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("Unable to create output directory: {e}"))?;
        }
    }

    let executable = detect_ffmpeg()
        .executable
        .ok_or_else(|| "FFmpeg was not found in PATH".to_string())?;

    let operation = operation.trim().to_ascii_lowercase();
    if !matches!(operation.as_str(), "video" | "compress" | "audio" | "extract-audio" | "remux") {
        return Err("Unknown media operation".into());
    }

    validate_codec(&video_codec)?;
    validate_codec(&audio_codec)?;

    let mut args = vec![
        "-hide_banner".to_string(),
        "-y".to_string(),
    ];

    if hardware_decode {
        args.extend(["-hwaccel".into(), "auto".into()]);
    }

    args.extend(["-i".into(), input]);

    match operation.as_str() {
        "remux" => {
            args.extend(["-c".into(), "copy".into()]);
        }
        "audio" | "extract-audio" => {
            args.push("-vn".into());
            args.extend(["-c:a".into(), audio_codec]);
        }
        "video" | "compress" => {
            args.extend(["-c:v".into(), video_codec]);
            args.extend(["-c:a".into(), audio_codec]);
            if operation == "compress" {
                args.extend(["-b:v".into(), "2500k".into(), "-maxrate".into(), "3500k".into(), "-bufsize".into(), "5000k".into()]);
            }

            let mut filters = Vec::new();
            if let (Some(width), Some(height)) = (width, height) {
                if width > 0 && height > 0 {
                    if width > 8192 || height > 8192 {
                        return Err("Output resolution is limited to 8192x8192".into());
                    }
                    filters.push(format!("scale={width}:{height}"));
                }
            }
            if !filters.is_empty() {
                args.extend(["-vf".into(), filters.join(",")]);
            }
            if let Some(fps) = fps {
                if fps > 0 {
                    if fps > 240 {
                        return Err("Output FPS is limited to 240".into());
                    }
                    args.extend(["-r".into(), fps.to_string()]);
                }
            }
        }
        _ => unreachable!(),
    }

    args.push(output.clone());

    let result = Command::new(executable)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute FFmpeg: {e}"))?;

    Ok(MediaResult {
        success: result.status.success(),
        exit_code: result.status.code(),
        stdout: String::from_utf8_lossy(&result.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&result.stderr).trim().to_string(),
        output_path: output,
    })
}


fn command_text(path: &Path, switch: &str) -> String {
    Command::new(path)
        .args(["-hide_banner", switch])
        .output()
        .ok()
        .map(|output| {
            let mut text = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            text.push_str(&String::from_utf8_lossy(&output.stderr).to_ascii_lowercase());
            text
        })
        .unwrap_or_default()
}

fn detect_common_codecs(encoders: &str, decoders: &str) -> Vec<String> {
    const CODECS: &[(&str, &[&str])] = &[
        ("H.264 / AVC", &["h264", "libx264", "h264_nvenc", "h264_qsv", "h264_amf"]),
        ("H.265 / HEVC", &["hevc", "libx265", "hevc_nvenc", "hevc_qsv", "hevc_amf"]),
        ("VP8", &["vp8", "libvpx"]),
        ("VP9", &["vp9", "libvpx-vp9"]),
        ("AV1", &["av1", "libaom-av1", "libsvtav1", "av1_nvenc", "av1_qsv", "av1_amf"]),
        ("MPEG-4", &["mpeg4"]),
        ("MPEG-2", &["mpeg2video"]),
        ("AAC", &["aac"]),
        ("MP3", &["mp3", "libmp3lame"]),
        ("Opus", &["opus", "libopus"]),
        ("Vorbis", &["vorbis", "libvorbis"]),
        ("FLAC", &["flac"]),
        ("PCM/WAV", &["pcm_s16le", "pcm_s24le", "pcm_f32le"]),
        ("ALAC", &["alac"]),
    ];

    CODECS.iter()
        .filter(|(_, aliases)| aliases.iter().any(|name| encoders.contains(name) || decoders.contains(name)))
        .map(|(name, _)| (*name).to_string())
        .collect()
}

fn detect_codec_names(path: &Path, switch: &str, needles: &[&str]) -> Vec<String> {
    Command::new(path)
        .args(["-hide_banner", switch])
        .output()
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter_map(|line| {
                    if !needles.iter().any(|needle| line.contains(needle)) {
                        return None;
                    }
                    line.split_whitespace().nth(1).map(str::to_string)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn validate_codec(codec: &str) -> Result<(), String> {
    if codec.is_empty() || codec.len() > 64 {
        return Err("Codec name must be between 1 and 64 characters".into());
    }
    if !codec.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')) {
        return Err("Codec name contains unsupported characters".into());
    }
    Ok(())
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}



pub fn stream_media(input:String,url:String,video_codec:String,rotate:Option<String>)->Result<MediaResult,String>{
    if !Path::new(&input).is_file(){return Err("Streaming input file does not exist".into());}
    if !(url.starts_with("rtmp://")||url.starts_with("rtmps://")||url.starts_with("srt://")){return Err("Stream URL must use rtmp://, rtmps://, or srt://".into());}
    validate_codec(&video_codec)?;
    let executable=detect_ffmpeg().executable.ok_or_else(||"FFmpeg was not found in PATH".to_string())?;
    let mut args=vec!["-hide_banner".into(),"-re".into(),"-i".into(),input,"-c:v".into(),video_codec,"-c:a".into(),"aac".into()];
    if let Some(mode)=rotate{
        let filter=match mode.as_str(){"left"=>"transpose=2","right"=>"transpose=1","flip"=>"hflip,vflip","none"=>"" ,_=>return Err("rotate must be left, right, flip, or none".into())};
        if !filter.is_empty(){args.extend(["-vf".into(),filter.into()]);}
    }
    if url.starts_with("srt://"){args.extend(["-f".into(),"mpegts".into()]);}else{args.extend(["-f".into(),"flv".into()]);}
    args.push(url.clone());
    let result=Command::new(executable).args(&args).output().map_err(|e|format!("Failed to execute FFmpeg stream: {e}"))?;
    Ok(MediaResult{success:result.status.success(),exit_code:result.status.code(),stdout:String::from_utf8_lossy(&result.stdout).trim().to_string(),stderr:String::from_utf8_lossy(&result.stderr).trim().to_string(),output_path:url})
}

pub fn run_media_batch(jobs: Vec<MediaJobRequest>) -> Result<Vec<MediaResult>, String> {
    if jobs.is_empty() {
        return Err("Media batch cannot be empty".into());
    }
    if jobs.len() > 100 {
        return Err("Media batch is limited to 100 jobs".into());
    }

    let mut results = Vec::with_capacity(jobs.len());
    for job in jobs {
        let result = run_media_job(
            job.input,
            job.output,
            job.operation,
            job.video_codec,
            job.audio_codec,
            job.width,
            job.height,
            job.fps,
            job.hardware_decode,
        )?;
        let failed = !result.success;
        results.push(result);
        if failed {
            break;
        }
    }
    Ok(results)
}
