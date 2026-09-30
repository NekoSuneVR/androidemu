use serde::Serialize;
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegInfo {
    pub found: bool,
    pub executable: Option<String>,
    pub version: Option<String>,
    pub hwaccels: Vec<String>,
    pub hardware_encoders: Vec<String>,
    pub hardware_decoders: Vec<String>,
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

    let executable = candidates.iter().find_map(|name| find_in_path(name));
    let Some(path) = executable else {
        return FfmpegInfo {
            found: false,
            executable: None,
            version: None,
            hwaccels: Vec::new(),
            hardware_encoders: Vec::new(),
            hardware_decoders: Vec::new(),
        };
    };

    let version = Command::new(&path)
        .arg("-version")
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(str::to_string)
        });

    let hwaccels = Command::new(&path)
        .args(["-hide_banner", "-hwaccels"])
        .output()
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

    FfmpegInfo {
        found: true,
        executable: Some(path.to_string_lossy().to_string()),
        version,
        hwaccels,
        hardware_encoders,
        hardware_decoders,
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
    if !matches!(operation.as_str(), "video" | "audio" | "extract-audio" | "remux") {
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
        "video" => {
            args.extend(["-c:v".into(), video_codec]);
            args.extend(["-c:a".into(), audio_codec]);

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
