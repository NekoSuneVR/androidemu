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
pub struct PrerequisiteInstallResult {
    pub success: bool,
    pub reboot_required: bool,
    pub messages: Vec<String>,
}

pub fn install_missing_windows_runtime() -> Result<PrerequisiteInstallResult, String> {
    if !cfg!(windows) {
        return Err("Automatic prerequisite installation is currently only available on Windows.".into());
    }

    let mut messages = Vec::new();
    let mut reboot_required = false;

    if runtime::find_runtime_tool("pacman").is_none() {
        messages.push("MSYS2 was not detected; installing MSYS2...".into());
        install_msys2(&mut messages)?;
    } else {
        messages.push("MSYS2 is already installed.".into());
    }

    let pacman = runtime::find_runtime_tool("pacman")
        .or_else(|| {
            let candidate = PathBuf::from(r"C:\msys64\usr\bin\pacman.exe");
            candidate.is_file().then_some(candidate)
        })
        .ok_or_else(|| "MSYS2 was installed, but pacman.exe could not be found. Restart NekoDroid and run Install/Repair again.".to_string())?;

    let packages = [
        "mingw-w64-ucrt-x86_64-qemu",
        "mingw-w64-ucrt-x86_64-qemu-image-util",
        "mingw-w64-ucrt-x86_64-android-tools",
        "mingw-w64-ucrt-x86_64-ffmpeg",
    ];

    messages.push("Updating MSYS2 package databases and runtime packages...".into());
    let update = run_msys2_command(&pacman, &["-Syu", "--noconfirm"])?;
    if !update.status.success() {
        let detail = output_text(&update);
        return Err(format!("MSYS2 update failed: {detail}"));
    }

    messages.push("Installing NekoDroid Windows runtime packages (QEMU, qemu-img, ADB and FFmpeg)...".into());
    let mut args = vec!["-S", "--needed", "--noconfirm"];
    args.extend(packages);
    let install = run_msys2_command(&pacman, &args)?;
    if !install.status.success() {
        let detail = output_text(&install);
        return Err(format!("MSYS2 package installation failed: {detail}"));
    }

    for (tool, label) in [
        ("qemu-system-x86_64", "QEMU x86_64 runtime"),
        ("qemu-img", "qemu-img"),
        ("adb", "Android platform tools / ADB"),
        ("ffmpeg", "FFmpeg"),
    ] {
        if let Some(path) = runtime::find_runtime_tool(tool) {
            messages.push(format!("{label}: {}", path.display()));
        } else {
            messages.push(format!("{label}: still not detected after installation."));
        }
    }

    if !windows_feature_enabled("HypervisorPlatform") {
        messages.push("Windows Hypervisor Platform is disabled. Requesting administrator approval to enable it...".into());
        enable_windows_feature_elevated("HypervisorPlatform")?;
        reboot_required = true;
        messages.push("Windows Hypervisor Platform enable command completed. A Windows restart may be required.".into());
    } else {
        messages.push("Windows Hypervisor Platform is already enabled.".into());
    }

    let firmware = powershell_output(
        "$p=Get-CimInstance Win32_Processor | Select-Object -First 1; if($p.VirtualizationFirmwareEnabled){'true'}else{'false'}"
    ).unwrap_or_default();
    if !firmware.trim().eq_ignore_ascii_case("true") {
        messages.push("CPU virtualization is not reported as enabled in firmware. If WHPX is still unavailable after reboot, enable Intel VT-x/AMD-V in BIOS/UEFI.".into());
    }

    let qemu_ok = runtime::find_runtime_tool("qemu-system-x86_64").is_some();
    let image_ok = runtime::find_runtime_tool("qemu-img").is_some();
    let adb_ok = runtime::find_runtime_tool("adb").is_some();

    Ok(PrerequisiteInstallResult {
        success: qemu_ok && image_ok && adb_ok,
        reboot_required,
        messages,
    })
}

fn install_msys2(messages: &mut Vec<String>) -> Result<(), String> {
    if let Some(winget) = find_in_path("winget.exe").or_else(|| find_in_path("winget")) {
        let output = Command::new(winget)
            .args([
                "install",
                "--id", "MSYS2.MSYS2",
                "--exact",
                "--silent",
                "--accept-package-agreements",
                "--accept-source-agreements",
                "--disable-interactivity",
            ])
            .output()
            .map_err(|e| format!("Unable to launch winget: {e}"))?;

        if output.status.success() || Path::new(r"C:\msys64\usr\bin\pacman.exe").is_file() {
            messages.push("MSYS2 installed through Windows Package Manager.".into());
            return Ok(());
        }

        messages.push(format!(
            "winget could not install MSYS2 ({}); trying the official MSYS2 installer.",
            output_text(&output)
        ));
    } else {
        messages.push("Windows Package Manager was not found; downloading the official MSYS2 installer.".into());
    }

    install_msys2_from_github(messages)
}

fn install_msys2_from_github(messages: &mut Vec<String>) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("NekoDroid prerequisite installer")
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let release: serde_json::Value = client
        .get("https://api.github.com/repos/msys2/msys2-installer/releases/latest")
        .send()
        .map_err(|e| format!("Unable to query the latest MSYS2 installer: {e}"))?
        .error_for_status()
        .map_err(|e| format!("MSYS2 release lookup failed: {e}"))?
        .json()
        .map_err(|e| format!("Invalid MSYS2 release response: {e}"))?;

    let asset = release
        .get("assets")
        .and_then(|v| v.as_array())
        .and_then(|assets| assets.iter().find(|asset| {
            asset.get("name").and_then(|v| v.as_str())
                .map(|name| name.starts_with("msys2-x86_64-") && name.ends_with(".exe"))
                .unwrap_or(false)
        }))
        .ok_or_else(|| "The latest MSYS2 release did not contain an x86_64 installer.".to_string())?;

    let url = asset.get("browser_download_url").and_then(|v| v.as_str())
        .ok_or_else(|| "The MSYS2 installer download URL was missing.".to_string())?;

    let bytes = client
        .get(url)
        .send()
        .map_err(|e| format!("Unable to download MSYS2: {e}"))?
        .error_for_status()
        .map_err(|e| format!("MSYS2 installer download failed: {e}"))?
        .bytes()
        .map_err(|e| e.to_string())?;

    let installer = env::temp_dir().join("nekodroid-msys2-installer.exe");
    fs::write(&installer, &bytes).map_err(|e| format!("Unable to save MSYS2 installer: {e}"))?;

    let status = Command::new(&installer)
        .args(["install", "--confirm-command", "--root", r"C:\msys64"])
        .status()
        .map_err(|e| format!("Unable to start the MSYS2 installer: {e}"))?;
    let _ = fs::remove_file(&installer);

    if !status.success() && !Path::new(r"C:\msys64\usr\bin\pacman.exe").is_file() {
        return Err(format!("MSYS2 installer exited with status {status}."));
    }

    messages.push("MSYS2 installed from the official MSYS2 GitHub release.".into());
    Ok(())
}

fn run_msys2_command(pacman: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    let mut command = Command::new(pacman);
    runtime::apply_runtime_environment(&mut command, pacman);
    command.args(args).output().map_err(|e| format!("Unable to run pacman: {e}"))
}

fn windows_feature_enabled(feature: &str) -> bool {
    let script = format!(
        "(Get-WindowsOptionalFeature -Online -FeatureName '{}').State",
        feature.replace('\'', "''")
    );
    powershell_output(&script)
        .map(|value| value.trim().eq_ignore_ascii_case("Enabled"))
        .unwrap_or(false)
}

fn enable_windows_feature_elevated(feature: &str) -> Result<(), String> {
    let script = format!(
        "$p=Start-Process -FilePath dism.exe -Verb RunAs -Wait -PassThru -ArgumentList @('/Online','/Enable-Feature','/FeatureName:{}','/All','/NoRestart'); exit $p.ExitCode",
        feature
    );
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .status()
        .map_err(|e| format!("Unable to request Windows administrator approval: {e}"))?;

    if !status.success() {
        return Err("Windows Hypervisor Platform was not enabled. The UAC prompt may have been cancelled.".into());
    }
    Ok(())
}

fn powershell_output(script: &str) -> Option<String> {
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn output_text(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() { stderr } else if !stdout.is_empty() { stdout } else { format!("exit code {:?}", output.status.code()) }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}
