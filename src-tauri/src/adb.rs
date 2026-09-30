use crate::models::{AdbInfo, AdbResult};
use serde::Serialize;
use std::collections::BTreeSet;
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApkCompatibility {
    pub path: String,
    pub abis: Vec<String>,
    pub preferred_abi: String,
    pub native_x86_64: bool,
    pub needs_arm_compatibility: bool,
    pub diagnostic: String,
}

pub fn inspect_apk(apk_path: String) -> Result<ApkCompatibility, String> {
    let path = Path::new(&apk_path);
    if !path.is_file() { return Err(format!("APK does not exist: {apk_path}")); }
    let file = fs::File::open(path).map_err(|e| format!("Unable to open APK: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid APK/ZIP: {e}"))?;
    let mut abis = BTreeSet::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let name = entry.name();
        if let Some(rest) = name.strip_prefix("lib/") {
            if let Some((abi, _)) = rest.split_once('/') {
                if !abi.is_empty() { abis.insert(abi.to_string()); }
            }
        }
    }
    let abis = abis.into_iter().collect::<Vec<_>>();
    let native_x86_64 = abis.iter().any(|abi| abi == "x86_64");
    let preferred_abi = if native_x86_64 { "x86_64".to_string() }
        else if abis.iter().any(|abi| abi == "x86") { "x86".to_string() }
        else if abis.iter().any(|abi| abi == "arm64-v8a") { "arm64-v8a".to_string() }
        else if abis.iter().any(|abi| abi == "armeabi-v7a") { "armeabi-v7a".to_string() }
        else { "no native libraries / universal Java-Kotlin".to_string() };
    let needs_arm_compatibility = !native_x86_64 && abis.iter().any(|abi| abi.starts_with("arm") || abi.starts_with("armeabi"));
    let diagnostic = if abis.is_empty() {
        "No ABI-specific native libraries found; the app may be architecture-neutral.".to_string()
    } else if native_x86_64 {
        "Native x86_64 libraries are present and should be preferred.".to_string()
    } else if needs_arm_compatibility {
        format!("No x86_64 libraries found. Preferred available ABI is {preferred_abi}; ARM compatibility may be required.")
    } else {
        format!("Available native ABIs: {}.", abis.join(", "))
    };
    Ok(ApkCompatibility { path: apk_path, abis, preferred_abi, native_x86_64, needs_arm_compatibility, diagnostic })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: String,
    pub modified: i64,
}

static ADB_KEY_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn configure_adb_keys(data_dir: &Path) -> Result<(), String> {
    let dir = data_dir.join("adb");
    fs::create_dir_all(&dir).map_err(|e| format!("Unable to create ADB key directory: {e}"))?;
    let key = dir.join("adbkey");
    if !key.is_file() {
        if let Some(adb) = detect_adb().executable {
            let output = Command::new(adb).args(["keygen", key.to_string_lossy().as_ref()]).output()
                .map_err(|e| format!("Unable to generate ADB key: {e}"))?;
            if !output.status.success() {
                return Err(format!("ADB key generation failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
            }
        }
    }
    #[cfg(unix)]
    if key.is_file() {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).map_err(|e| format!("Unable to secure ADB key permissions: {e}"))?;
    }
    let _ = ADB_KEY_PATH.set(key);
    Ok(())
}

pub fn detect_adb() -> AdbInfo {
    let candidates: &[&str] = if cfg!(windows) {
        &["adb.exe", "adb"]
    } else {
        &["adb"]
    };

    let executable = candidates.iter().find_map(|name| find_in_path(name));

    let Some(path) = executable else {
        return AdbInfo {
            found: false,
            executable: None,
            version: None,
        };
    };

    let version = Command::new(&path)
        .arg("version")
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or_default().to_string())
        .filter(|value| !value.is_empty());

    AdbInfo {
        found: true,
        executable: Some(path.to_string_lossy().to_string()),
        version,
    }
}

pub fn connect(port: u16) -> Result<AdbResult, String> {
    run_adb(&["connect".into(), format!("127.0.0.1:{port}")])
}

pub fn disconnect(port: u16) -> Result<AdbResult, String> {
    run_adb(&["disconnect".into(), format!("127.0.0.1:{port}")])
}

pub fn root(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &["root".into()])
}

pub fn unroot(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &["unroot".into()])
}

pub fn root_shell(port: u16, command: String) -> Result<AdbResult, String> {
    if command.trim().is_empty() {
        return Err("Root shell command cannot be empty".into());
    }
    run_for_device(port, &[
        "shell".into(),
        "su".into(),
        "-c".into(),
        command,
    ])
}

pub fn get_state(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &["get-state".into()])
}

pub fn shell(port: u16, command: String) -> Result<AdbResult, String> {
    if command.trim().is_empty() {
        return Err("ADB shell command cannot be empty".into());
    }
    run_for_device(port, &["shell".into(), "sh".into(), "-c".into(), command])
}

pub fn install(port: u16, apk_path: String) -> Result<AdbResult, String> {
    if !Path::new(&apk_path).is_file() {
        return Err(format!("APK does not exist: {apk_path}"));
    }
    run_for_device(port, &["install".into(), "-r".into(), apk_path])
}

pub fn install_batch(port: u16, apk_paths: Vec<String>) -> Result<AdbResult, String> {
    if apk_paths.is_empty() {
        return Err("Provide at least one APK path".into());
    }

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut success = true;
    let mut exit_code = Some(0);

    for apk_path in apk_paths {
        let result = install(port, apk_path.clone())?;
        stdout.push(format!("{apk_path}: {}", result.stdout));
        if !result.stderr.is_empty() {
            stderr.push(format!("{apk_path}: {}", result.stderr));
        }
        if !result.success {
            success = false;
            exit_code = result.exit_code;
        }
    }

    Ok(AdbResult {
        success,
        exit_code,
        stdout: stdout.join("\n"),
        stderr: stderr.join("\n"),
    })
}

pub fn install_multiple(port: u16, apk_paths: Vec<String>) -> Result<AdbResult, String> {
    if apk_paths.len() < 2 {
        return Err("Split APK installation requires at least two APK files".into());
    }

    let mut args = vec!["install-multiple".into(), "-r".into()];
    for apk_path in apk_paths {
        if !Path::new(&apk_path).is_file() {
            return Err(format!("APK does not exist: {apk_path}"));
        }
        args.push(apk_path);
    }
    run_for_device(port, &args)
}

pub fn uninstall(port: u16, package_name: String) -> Result<AdbResult, String> {
    let package_name = package_name.trim();
    if package_name.is_empty() {
        return Err("Package name cannot be empty".into());
    }
    run_for_device(port, &["uninstall".into(), package_name.into()])
}

pub fn shutdown(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &["shell".into(), "reboot".into(), "-p".into()])
}

pub fn boot_status(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(), "sh".into(), "-c".into(),
        "printf 'sys.boot_completed='; getprop sys.boot_completed; printf 'bootanim='; getprop init.svc.bootanim; printf 'launcher='; dumpsys window windows 2>/dev/null | grep -E 'mCurrentFocus|mFocusedApp' | head -n 1".into()
    ])
}

pub fn crash_diagnostics(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(), "sh".into(), "-c".into(),
        "echo '--- crash log ---'; logcat -b crash -d -t 200 2>/dev/null; echo '--- tombstones ---'; ls -lt /data/tombstones 2>/dev/null | head -n 20; echo '--- last ANRs ---'; ls -lt /data/anr 2>/dev/null | head -n 20".into()
    ])
}

pub fn reboot(port: u16, mode: Option<String>) -> Result<AdbResult, String> {
    let mut args = vec!["reboot".into()];
    if let Some(mode) = mode {
        let mode = mode.trim().to_ascii_lowercase();
        if !mode.is_empty() {
            if !matches!(mode.as_str(), "recovery" | "bootloader") {
                return Err("Reboot mode must be recovery or bootloader".into());
            }
            args.push(mode);
        }
    }
    run_for_device(port, &args)
}

pub fn set_lan_adb(port:u16,enabled:bool,lan_port:u16)->Result<AdbResult,String>{
    if lan_port==0{return Err("LAN ADB port must be 1..65535".into());}
    let command=if enabled{
        format!("setprop service.adb.tcp.port {lan_port}; stop adbd; start adbd; echo 'ADB LAN enabled on port {lan_port}'; ip -4 addr show 2>/dev/null | grep -E 'inet '")
    }else{
        "setprop service.adb.tcp.port -1; stop adbd; start adbd; echo 'ADB LAN disabled'".into()
    };
    root_shell(port,command)
}

pub fn clipboard_set(port: u16, text: String) -> Result<AdbResult, String> {
    if text.len() > 100_000 { return Err("Clipboard text is limited to 100,000 characters".into()); }
    run_for_device(port, &["shell".into(), "cmd".into(), "clipboard".into(), "set".into(), text])
}

pub fn clipboard_get(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &["shell".into(), "cmd".into(), "clipboard".into(), "get".into()])
}

pub fn media_codec_requests(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(), "sh".into(), "-c".into(),
        "echo '--- codec services ---'; dumpsys media.codec 2>/dev/null | head -n 300; echo '--- recent codec log ---'; logcat -d -t 500 2>/dev/null | grep -Ei 'MediaCodec|Codec2|CCodec|OMX' | tail -n 200".into()
    ])
}

pub fn security_state(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(),
        "sh".into(),
        "-c".into(),
        "echo 'verified_boot_state='$(getprop ro.boot.verifiedbootstate); echo 'vbmeta_device_state='$(getprop ro.boot.vbmeta.device_state); echo 'flash_locked='$(getprop ro.boot.flash.locked); echo 'build_tags='$(getprop ro.build.tags); echo 'product_model='$(getprop ro.product.model); echo 'hardware_attestation=not asserted by NekoDroid'".into(),
    ])
}

pub fn device_info(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(),
        "sh".into(),
        "-c".into(),
        "getprop && echo && echo '--- storage ---' && df -h /data /sdcard 2>/dev/null || true".into(),
    ])
}

pub fn screenshot(port: u16, destination: String) -> Result<AdbResult, String> {
    if destination.trim().is_empty() {
        return Err("Screenshot destination cannot be empty".into());
    }

    let adb = detect_adb()
        .executable
        .ok_or_else(|| "ADB was not found in PATH. Install Android platform-tools or configure it for NekoDroid.".to_string())?;

    let output = Command::new(adb)
        .args([
            "-s",
            &format!("127.0.0.1:{port}"),
            "exec-out",
            "screencap",
            "-p",
        ])
        .output()
        .map_err(|e| format!("Failed to execute ADB screenshot: {e}"))?;

    if !output.status.success() {
        return Ok(AdbResult {
            success: false,
            exit_code: output.status.code(),
            stdout: String::new(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    fs::write(&destination, output.stdout)
        .map_err(|e| format!("Failed to write screenshot to {destination}: {e}"))?;

    Ok(AdbResult {
        success: true,
        exit_code: output.status.code(),
        stdout: format!("Screenshot saved to {destination}"),
        stderr: String::new(),
    })
}

pub fn screen_record(
    port: u16,
    destination: String,
    seconds: u32,
) -> Result<AdbResult, String> {
    if destination.trim().is_empty() {
        return Err("Recording destination cannot be empty".into());
    }
    let seconds = seconds.clamp(1, 180);
    let remote_path = "/sdcard/Download/nekodroid-recording.mp4";

    let record = run_for_device(port, &[
        "shell".into(),
        "screenrecord".into(),
        "--time-limit".into(),
        seconds.to_string(),
        remote_path.into(),
    ])?;
    if !record.success {
        return Ok(record);
    }

    let pulled = pull(port, remote_path.into(), destination.clone())?;
    let _ = run_for_device(port, &["shell".into(), "rm".into(), "-f".into(), remote_path.into()]);

    Ok(AdbResult {
        success: pulled.success,
        exit_code: pulled.exit_code,
        stdout: if pulled.success {
            format!("Screen recording saved to {destination}")
        } else {
            pulled.stdout
        },
        stderr: pulled.stderr,
    })
}


pub fn screen_record_advanced(
    port:u16,
    destination:String,
    seconds:u32,
    codec:String,
    bitrate_mbps:u32,
    fps:Option<u32>,
    audio:bool,
)->Result<AdbResult,String>{
    if destination.trim().is_empty(){return Err("Recording destination cannot be empty".into());}
    let codec=codec.to_ascii_lowercase();
    if !matches!(codec.as_str(),"h264"|"hevc"){return Err("Recording codec must be h264 or hevc".into());}
    if let Some(fps)=fps{let _=set_refresh_rate(port,Some(fps));}
    let seconds=seconds.clamp(1,180);
    let bitrate=bitrate_mbps.clamp(1,100)*1_000_000;
    let remote="/sdcard/Download/nekodroid-recording.mp4";
    let mut args=vec!["shell".into(),"screenrecord".into(),"--time-limit".into(),seconds.to_string(),"--bit-rate".into(),bitrate.to_string(),"--codec".into(),codec,];
    if audio{args.push("--audio".into());}
    args.push(remote.into());
    let record=run_for_device(port,&args)?;
    if !record.success{return Ok(record);}
    let pulled=pull(port,remote.into(),destination.clone())?;
    let _=run_for_device(port,&["shell".into(),"rm".into(),"-f".into(),remote.into()]);
    Ok(AdbResult{success:pulled.success,exit_code:pulled.exit_code,stdout:if pulled.success{format!("Advanced recording saved to {destination}")}else{pulled.stdout},stderr:pulled.stderr})
}

pub fn push(port: u16, source: String, destination: String) -> Result<AdbResult, String> {
    if !std::path::Path::new(&source).exists() {
        return Err(format!("Source does not exist: {source}"));
    }
    if destination.trim().is_empty() {
        return Err("Android destination cannot be empty".into());
    }
    run_for_device(port, &["push".into(), source, destination])
}

pub fn push_multiple(port: u16, sources: Vec<String>, destination: String) -> Result<AdbResult, String> {
    if sources.is_empty() {
        return Err("Provide at least one host source path".into());
    }
    if sources.len() > 100 {
        return Err("Multiple-file transfer is limited to 100 sources".into());
    }
    if destination.trim().is_empty() {
        return Err("Android destination cannot be empty".into());
    }

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut success = true;
    let mut exit_code = Some(0);

    for source in sources {
        let result = push(port, source.clone(), destination.clone())?;
        stdout.push(format!("{source}: {}", result.stdout));
        if !result.stderr.is_empty() {
            stderr.push(format!("{source}: {}", result.stderr));
        }
        if !result.success {
            success = false;
            exit_code = result.exit_code;
            break;
        }
    }

    Ok(AdbResult {
        success,
        exit_code,
        stdout: stdout.join("\n"),
        stderr: stderr.join("\n"),
    })
}

pub fn pull(port: u16, source: String, destination: String) -> Result<AdbResult, String> {
    if source.trim().is_empty() {
        return Err("Android source cannot be empty".into());
    }
    if destination.trim().is_empty() {
        return Err("Host destination cannot be empty".into());
    }
    run_for_device(port, &["pull".into(), source, destination])
}

pub fn forward(port: u16, local: String, remote: String) -> Result<AdbResult, String> {
    validate_socket_spec(&local, "Local forward")?;
    validate_socket_spec(&remote, "Remote forward")?;
    run_for_device(port, &["forward".into(), local, remote])
}

pub fn reverse(port: u16, remote: String, local: String) -> Result<AdbResult, String> {
    validate_socket_spec(&remote, "Remote reverse")?;
    validate_socket_spec(&local, "Local reverse")?;
    run_for_device(port, &["reverse".into(), remote, local])
}

pub fn logcat(port: u16, lines: u32) -> Result<AdbResult, String> {
    let lines = lines.clamp(1, 5000);
    run_for_device(port, &[
        "logcat".into(),
        "-d".into(),
        "-t".into(),
        lines.to_string(),
    ])
}

pub fn packages(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &["shell".into(), "pm".into(), "list".into(), "packages".into(), "-f".into()])
}

pub fn user_packages(port: u16) -> Result<Vec<String>, String> {
    let result = run_for_device(port, &[
        "shell".into(),
        "pm".into(),
        "list".into(),
        "packages".into(),
        "-3".into(),
    ])?;

    if !result.success {
        return Err(if result.stderr.is_empty() { result.stdout } else { result.stderr });
    }

    let mut packages = result.stdout
        .lines()
        .filter_map(|line| line.trim().strip_prefix("package:"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    packages.sort();
    packages.dedup();
    Ok(packages)
}

pub fn launch_package(port: u16, package_name: String) -> Result<AdbResult, String> {
    validate_package_name(&package_name)?;
    run_for_device(port, &[
        "shell".into(),
        "monkey".into(),
        "-p".into(),
        package_name,
        "-c".into(),
        "android.intent.category.LAUNCHER".into(),
        "1".into(),
    ])
}

pub fn processes(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &["shell".into(), "ps".into(), "-A".into()])
}

pub fn properties(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &["shell".into(), "getprop".into()])
}

pub fn build_properties(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(),
        "sh".into(),
        "-c".into(),
        "getprop | grep -E 'ro\\.(build|product|system|vendor|bootimage|hardware|soc|cpu)'".into(),
    ])
}

pub fn storage_info(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(),
        "sh".into(),
        "-c".into(),
        "df -h /data /sdcard /storage/emulated/0 2>/dev/null; echo; du -sh /sdcard 2>/dev/null || true".into(),
    ])
}

pub fn activities(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(),
        "dumpsys".into(),
        "activity".into(),
        "activities".into(),
    ])
}

pub fn services(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(),
        "service".into(),
        "list".into(),
    ])
}

pub fn surfaceflinger_info(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(),
        "dumpsys".into(),
        "SurfaceFlinger".into(),
    ])
}

pub fn network_connections(port: u16) -> Result<AdbResult, String> {
    run_for_device(port, &[
        "shell".into(),
        "sh".into(),
        "-c".into(),
        "ss -tunap 2>/dev/null || netstat -tunap 2>/dev/null || cat /proc/net/tcp /proc/net/tcp6 /proc/net/udp /proc/net/udp6".into(),
    ])
}

pub fn kernel_log(port: u16, lines: u32) -> Result<AdbResult, String> {
    let lines = lines.clamp(1, 5000);
    run_for_device(port, &[
        "shell".into(),
        "sh".into(),
        "-c".into(),
        format!("dmesg | tail -n {lines}"),
    ])
}

pub fn input_tap(port: u16, x: i32, y: i32) -> Result<AdbResult, String> {
    validate_coord(x)?;
    validate_coord(y)?;
    run_for_device(port, &[
        "shell".into(), "input".into(), "tap".into(), x.to_string(), y.to_string()
    ])
}

pub fn input_double_tap(port: u16, x: i32, y: i32) -> Result<AdbResult, String> {
    validate_coord(x)?;
    validate_coord(y)?;
    let first = run_for_device(port, &[
        "shell".into(), "input".into(), "tap".into(), x.to_string(), y.to_string()
    ])?;
    if !first.success {
        return Ok(first);
    }
    std::thread::sleep(std::time::Duration::from_millis(120));
    run_for_device(port, &[
        "shell".into(), "input".into(), "tap".into(), x.to_string(), y.to_string()
    ])
}

pub fn input_hold(port: u16, x: i32, y: i32, duration_ms: u32) -> Result<AdbResult, String> {
    input_swipe(port, x, y, x, y, duration_ms)
}

pub fn input_swipe(
    port: u16,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    duration_ms: u32,
) -> Result<AdbResult, String> {
    validate_coord(x1)?;
    validate_coord(y1)?;
    validate_coord(x2)?;
    validate_coord(y2)?;
    let duration_ms = duration_ms.clamp(50, 5000);
    run_for_device(port, &[
        "shell".into(), "input".into(), "swipe".into(),
        x1.to_string(), y1.to_string(), x2.to_string(), y2.to_string(),
        duration_ms.to_string(),
    ])
}

pub fn input_keyevent(port: u16, keycode: String) -> Result<AdbResult, String> {
    const ALLOWED: &[&str] = &[
        "KEYCODE_BACK", "KEYCODE_HOME", "KEYCODE_APP_SWITCH", "KEYCODE_ENTER",
        "KEYCODE_DEL", "KEYCODE_DPAD_UP", "KEYCODE_DPAD_DOWN",
        "KEYCODE_DPAD_LEFT", "KEYCODE_DPAD_RIGHT", "KEYCODE_ESCAPE",
    ];
    let keycode = keycode.trim().to_ascii_uppercase();
    if !ALLOWED.contains(&keycode.as_str()) {
        return Err("Unsupported Android keycode".into());
    }
    run_for_device(port, &[
        "shell".into(), "input".into(), "keyevent".into(), keycode
    ])
}

pub fn input_text(port: u16, text: String) -> Result<AdbResult, String> {
    if text.is_empty() {
        return Err("Input text cannot be empty".into());
    }
    if text.len() > 128 {
        return Err("Input text is limited to 128 bytes".into());
    }
    if !text.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
        return Err("Input text currently supports printable ASCII only".into());
    }

    let encoded = text.replace('%', "%25").replace(' ', "%s");
    run_for_device(port, &[
        "shell".into(), "input".into(), "text".into(), encoded
    ])
}

fn validate_coord(value: i32) -> Result<(), String> {
    if !(0..=32767).contains(&value) {
        return Err("Input coordinate must be between 0 and 32767".into());
    }
    Ok(())
}

pub fn set_refresh_rate(port: u16, fps: Option<u32>) -> Result<AdbResult, String> {
    match fps {
        Some(value) => {
            if !matches!(value, 30 | 60 | 90 | 120 | 144 | 165 | 240) {
                return Err("FPS must be one of 30, 60, 90, 120, 144, 165, or 240".into());
            }
            run_for_device(port, &[
                "shell".into(), "sh".into(), "-c".into(),
                format!("settings put system min_refresh_rate {value}; settings put system peak_refresh_rate {value}; settings put system user_refresh_rate {value}")
            ])
        }
        None => run_for_device(port, &[
            "shell".into(), "sh".into(), "-c".into(),
            "settings delete system min_refresh_rate; settings delete system peak_refresh_rate; settings delete system user_refresh_rate".into()
        ])
    }
}

pub fn set_orientation(port: u16, orientation: String) -> Result<AdbResult, String> {
    let orientation = orientation.trim().to_ascii_lowercase();

    if orientation == "auto" {
        return run_for_device(port, &[
            "shell".into(),
            "settings".into(),
            "put".into(),
            "system".into(),
            "accelerometer_rotation".into(),
            "1".into(),
        ]);
    }

    let rotation = match orientation.as_str() {
        "portrait" => "0",
        "landscape" => "1",
        "reverse-portrait" => "2",
        "reverse-landscape" => "3",
        _ => return Err("Orientation must be auto, portrait, landscape, reverse-portrait, or reverse-landscape".into()),
    };

    let disable_auto = run_for_device(port, &[
        "shell".into(),
        "settings".into(),
        "put".into(),
        "system".into(),
        "accelerometer_rotation".into(),
        "0".into(),
    ])?;
    if !disable_auto.success {
        return Ok(disable_auto);
    }

    run_for_device(port, &[
        "shell".into(),
        "settings".into(),
        "put".into(),
        "system".into(),
        "user_rotation".into(),
        rotation.into(),
    ])
}

pub fn rotate_orientation(port: u16, direction: String) -> Result<AdbResult, String> {
    let direction = direction.trim().to_ascii_lowercase();
    if !matches!(direction.as_str(), "left" | "right") {
        return Err("Rotation direction must be left or right".into());
    }

    let current = run_for_device(port, &[
        "shell".into(),
        "settings".into(),
        "get".into(),
        "system".into(),
        "user_rotation".into(),
    ])?;
    if !current.success {
        return Ok(current);
    }

    let current_value = current.stdout.trim().parse::<i32>().unwrap_or(0).rem_euclid(4);
    let next = if direction == "left" {
        (current_value + 3).rem_euclid(4)
    } else {
        (current_value + 1).rem_euclid(4)
    };

    let disable_auto = run_for_device(port, &[
        "shell".into(),
        "settings".into(),
        "put".into(),
        "system".into(),
        "accelerometer_rotation".into(),
        "0".into(),
    ])?;
    if !disable_auto.success {
        return Ok(disable_auto);
    }

    run_for_device(port, &[
        "shell".into(),
        "settings".into(),
        "put".into(),
        "system".into(),
        "user_rotation".into(),
        next.to_string(),
    ])
}


pub fn list_files(port: u16, path: String) -> Result<Vec<AndroidFileEntry>, String> {
    validate_android_path(&path)?;
    let listing = run_for_device(port, &[
        "shell".into(), "ls".into(), "-1A".into(), path.clone()
    ])?;
    if !listing.success {
        return Err(if listing.stderr.is_empty() { listing.stdout } else { listing.stderr });
    }

    let mut entries = Vec::new();
    for name in listing.stdout.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if name == "." || name == ".." {
            continue;
        }
        let full_path = join_android_path(&path, name);
        let stat = run_for_device(port, &[
            "shell".into(),
            "stat".into(),
            "-c".into(),
            "%F|%s|%a|%Y".into(),
            full_path.clone(),
        ])?;

        let mut parts = stat.stdout.trim().split('|');
        let kind = parts.next().unwrap_or_default();
        let size = parts.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        let permissions = parts.next().unwrap_or_default().to_string();
        let modified = parts.next().and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);

        entries.push(AndroidFileEntry {
            name: name.to_string(),
            path: full_path,
            is_dir: kind.contains("directory"),
            size,
            permissions,
            modified,
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

pub fn make_directory(port: u16, path: String) -> Result<AdbResult, String> {
    validate_android_path(&path)?;
    run_for_device(port, &["shell".into(), "mkdir".into(), "-p".into(), path])
}

pub fn remove_path(port: u16, path: String) -> Result<AdbResult, String> {
    validate_android_path(&path)?;
    if path == "/" || path == "/system" || path == "/vendor" || path == "/data" {
        return Err("Refusing to recursively remove a protected top-level Android path".into());
    }
    run_for_device(port, &["shell".into(), "rm".into(), "-rf".into(), path])
}

pub fn move_path(port: u16, source: String, destination: String) -> Result<AdbResult, String> {
    validate_android_path(&source)?;
    validate_android_path(&destination)?;
    run_for_device(port, &["shell".into(), "mv".into(), source, destination])
}

pub fn copy_path(port: u16, source: String, destination: String) -> Result<AdbResult, String> {
    validate_android_path(&source)?;
    validate_android_path(&destination)?;
    run_for_device(port, &["shell".into(), "cp".into(), "-r".into(), source, destination])
}

pub fn file_properties(port: u16, path: String) -> Result<AdbResult, String> {
    validate_android_path(&path)?;
    run_for_device(port, &[
        "shell".into(),
        "stat".into(),
        "-c".into(),
        "Path: %n\\nType: %F\\nSize: %s bytes\\nPermissions: %A (%a)\\nOwner: %U:%G\\nModified: %y".into(),
        path,
    ])
}

pub fn search_files(port: u16, path: String, query: String) -> Result<AdbResult, String> {
    validate_android_path(&path)?;
    let query = query.trim();
    if query.is_empty() || query.len() > 128 {
        return Err("Search query must be between 1 and 128 characters".into());
    }
    run_for_device(port, &[
        "shell".into(),
        "find".into(),
        path,
        "-maxdepth".into(),
        "5".into(),
        "-iname".into(),
        format!("*{query}*"),
    ])
}

fn validate_package_name(package_name: &str) -> Result<(), String> {
    if package_name.is_empty()
        || package_name.len() > 255
        || !package_name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_'))
    {
        return Err("Invalid Android package name".into());
    }
    Ok(())
}

fn validate_android_path(path: &str) -> Result<(), String> {
    if !path.starts_with('/') {
        return Err("Android path must be absolute and start with '/'".into());
    }
    if path.contains('\0') || path.contains('\n') || path.contains('\r') {
        return Err("Android path contains invalid characters".into());
    }
    Ok(())
}

fn join_android_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), name)
    }
}

fn validate_socket_spec(value: &str, label: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label} cannot be empty"));
    }
    let valid = value.starts_with("tcp:")
        || value.starts_with("localabstract:")
        || value.starts_with("localfilesystem:")
        || value.starts_with("localreserved:");
    if !valid {
        return Err(format!("{label} must use tcp: or a local* ADB socket spec"));
    }
    Ok(())
}

fn run_for_device(port: u16, args: &[String]) -> Result<AdbResult, String> {
    let mut full = vec!["-s".into(), format!("127.0.0.1:{port}")];
    full.extend_from_slice(args);
    run_adb(&full)
}

fn run_adb(args: &[String]) -> Result<AdbResult, String> {
    let adb = detect_adb()
        .executable
        .ok_or_else(|| "ADB was not found in PATH. Install Android platform-tools or configure it for NekoDroid.".to_string())?;

    let mut command = Command::new(adb);
    command.args(args);
    if let Some(key) = ADB_KEY_PATH.get().filter(|path| path.is_file()) {
        command.env("ADB_VENDOR_KEYS", key);
    }
    let output = command
        .output()
        .map_err(|e| format!("Failed to execute ADB: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    Ok(AdbResult {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout,
        stderr,
    })
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}
