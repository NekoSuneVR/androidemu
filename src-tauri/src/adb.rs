use crate::models::{AdbInfo, AdbResult};
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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

pub fn uninstall(port: u16, package_name: String) -> Result<AdbResult, String> {
    let package_name = package_name.trim();
    if package_name.is_empty() {
        return Err("Package name cannot be empty".into());
    }
    run_for_device(port, &["uninstall".into(), package_name.into()])
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

pub fn push(port: u16, source: String, destination: String) -> Result<AdbResult, String> {
    if !std::path::Path::new(&source).exists() {
        return Err(format!("Source does not exist: {source}"));
    }
    if destination.trim().is_empty() {
        return Err("Android destination cannot be empty".into());
    }
    run_for_device(port, &["push".into(), source, destination])
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

    let output = Command::new(adb)
        .args(args)
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
