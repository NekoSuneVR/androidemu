use crate::{adb, media, runtime};
use serde::Serialize;
use std::{
    env,
    fs,
    path::PathBuf,
    process::Command,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemReadiness {
    pub cpu_model: String,
    pub logical_cores: usize,
    pub total_memory_mb: Option<u64>,
    pub virtualization_available: bool,
    pub virtualization_detail: String,
    pub gpu_names: Vec<String>,
    pub vulkan_available: bool,
    pub vulkan_detail: String,
    pub opengl_available: bool,
    pub opengl_detail: String,
    pub directx_available: bool,
    pub directx_detail: String,
    pub hardware_encoders: Vec<String>,
    pub free_disk_mb: Option<u64>,
    pub qemu_found: bool,
    pub adb_found: bool,
    pub ffmpeg_found: bool,
    pub recommended_cpu_cores: u16,
    pub recommended_ram_mb: u32,
    pub recommended_android_version: String,
    pub recommended_renderer: String,
}

pub fn detect() -> SystemReadiness {
    let cpu_model = detect_cpu_model();
    let logical_cores = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1);
    let total_memory_mb = detect_total_memory_mb();
    let (virtualization_available, virtualization_detail) = detect_virtualization();
    let gpu_names = detect_gpus();
    let (vulkan_available, vulkan_detail) = detect_vulkan();
    let (opengl_available, opengl_detail) = detect_opengl();
    let (directx_available, directx_detail) = detect_directx();
    let hardware_encoders = detect_hardware_encoders();
    let free_disk_mb = detect_free_disk_mb();
    let qemu = runtime::detect_qemu();
    let adb_info = adb::detect_adb();
    let ffmpeg = media::detect_ffmpeg();
    let recommended_renderer = if vulkan_available {
        "Vulkan".to_string()
    } else if !gpu_names.is_empty() {
        "Compatibility / OpenGL".to_string()
    } else {
        "Software".to_string()
    };

    let recommended_cpu_cores = logical_cores
        .saturating_sub(2)
        .clamp(2, 8) as u16;

    let recommended_ram_mb = total_memory_mb
        .map(|total| {
            let quarter = total / 4;
            quarter.clamp(2048, 8192) as u32
        })
        .unwrap_or(4096);

    SystemReadiness {
        cpu_model,
        logical_cores,
        total_memory_mb,
        virtualization_available,
        virtualization_detail,
        gpu_names,
        vulkan_available,
        vulkan_detail,
        opengl_available,
        opengl_detail,
        directx_available,
        directx_detail,
        hardware_encoders,
        free_disk_mb,
        qemu_found: qemu.found,
        adb_found: adb_info.found,
        ffmpeg_found: ffmpeg.found,
        recommended_cpu_cores,
        recommended_ram_mb,
        recommended_android_version: "16".into(),
        recommended_renderer,
    }
}

fn detect_cpu_model() -> String {
    if cfg!(target_os = "linux") {
        if let Ok(text) = fs::read_to_string("/proc/cpuinfo") {
            if let Some(line) = text.lines().find(|line| line.starts_with("model name")) {
                if let Some((_, value)) = line.split_once(':') {
                    return value.trim().to_string();
                }
            }
        }
    }

    if cfg!(windows) {
        if let Some(output) = powershell("(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name)") {
            if !output.is_empty() {
                return output;
            }
        }
        if let Ok(value) = env::var("PROCESSOR_IDENTIFIER") {
            return value;
        }
    }

    format!("{} {}", env::consts::ARCH, env::consts::OS)
}

fn detect_total_memory_mb() -> Option<u64> {
    if cfg!(target_os = "linux") {
        let text = fs::read_to_string("/proc/meminfo").ok()?;
        let line = text.lines().find(|line| line.starts_with("MemTotal:"))?;
        let kb = line.split_whitespace().nth(1)?.parse::<u64>().ok()?;
        return Some(kb / 1024);
    }

    if cfg!(windows) {
        let output = powershell("(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory")?;
        let bytes = output.trim().parse::<u64>().ok()?;
        return Some(bytes / 1024 / 1024);
    }

    None
}

fn detect_virtualization() -> (bool, String) {
    if cfg!(target_os = "linux") {
        let flags = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let cpu_support = flags.split_whitespace().any(|flag| flag == "vmx" || flag == "svm");
        let kvm = std::path::Path::new("/dev/kvm").exists();
        return (
            cpu_support && kvm,
            if !cpu_support {
                "CPU virtualization flag (VT-x/AMD-V) was not detected.".into()
            } else if !kvm {
                "CPU virtualization is supported, but /dev/kvm is unavailable.".into()
            } else {
                "CPU virtualization and /dev/kvm are available.".into()
            },
        );
    }

    if cfg!(windows) {
        let firmware = powershell(
            "$p=Get-CimInstance Win32_Processor | Select-Object -First 1; if($p.VirtualizationFirmwareEnabled){'true'}else{'false'}"
        ).unwrap_or_default();
        let qemu = runtime::detect_qemu();
        let whpx = qemu.accelerators.iter().any(|item| item == "whpx");
        let available = firmware.trim().eq_ignore_ascii_case("true") && whpx;
        return (
            available,
            format!(
                "Firmware virtualization: {}; QEMU WHPX: {}.",
                if firmware.trim().eq_ignore_ascii_case("true") { "enabled" } else { "not detected" },
                if whpx { "available" } else { "not advertised" }
            ),
        );
    }

    (false, "Virtualization detection is not implemented for this host OS yet.".into())
}

fn detect_gpus() -> Vec<String> {
    if cfg!(windows) {
        if let Some(output) = powershell("Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name") {
            let values = output.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect::<Vec<_>>();
            if !values.is_empty() {
                return values;
            }
        }
    }

    if cfg!(target_os = "linux") {
        if let Some(path) = find_in_path("lspci") {
            if let Ok(output) = Command::new(path).output() {
                let text = String::from_utf8_lossy(&output.stdout);
                let values = text.lines()
                    .filter(|line| line.contains("VGA compatible controller") || line.contains("3D controller") || line.contains("Display controller"))
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                if !values.is_empty() {
                    return values;
                }
            }
        }
    }

    Vec::new()
}

fn detect_vulkan() -> (bool, String) {
    let names: &[&str] = if cfg!(windows) {
        &["vulkaninfo.exe", "vulkaninfo"]
    } else {
        &["vulkaninfo"]
    };

    if let Some(path) = names.iter().find_map(|name| find_in_path(name)) {
        if let Ok(output) = Command::new(path).arg("--summary").output() {
            let success = output.status.success();
            let mut text = String::from_utf8_lossy(&output.stdout).to_string();
            if text.trim().is_empty() {
                text = String::from_utf8_lossy(&output.stderr).to_string();
            }
            let summary = text.lines().find(|line| !line.trim().is_empty()).unwrap_or("vulkaninfo executed").trim().to_string();
            return (success, summary);
        }
    }

    (false, "vulkaninfo was not found in PATH.".into())
}


fn detect_opengl() -> (bool, String) {
    if cfg!(target_os = "linux") {
        if let Some(path) = find_in_path("glxinfo") {
            if let Ok(output) = Command::new(path).arg("-B").output() {
                let mut text = String::from_utf8_lossy(&output.stdout).to_string();
                text.push_str(&String::from_utf8_lossy(&output.stderr));
                let version = text.lines()
                    .find(|line| line.to_ascii_lowercase().contains("opengl version"))
                    .map(str::trim)
                    .unwrap_or("glxinfo reported OpenGL support");
                return (output.status.success(), version.to_string());
            }
        }
        return (false, "glxinfo was not found in PATH.".into());
    }

    if cfg!(windows) {
        let system_root = env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        let dll = PathBuf::from(system_root).join("System32").join("opengl32.dll");
        return (
            dll.is_file(),
            if dll.is_file() {
                format!("Windows OpenGL runtime detected at {}.", dll.display())
            } else {
                "Windows OpenGL runtime was not detected.".into()
            },
        );
    }

    (false, "OpenGL host detection is not implemented for this host OS yet.".into())
}

fn detect_directx() -> (bool, String) {
    if !cfg!(windows) {
        return (false, "DirectX is only available on Windows hosts.".into());
    }

    let system_root = env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    let system32 = PathBuf::from(system_root).join("System32");
    let d3d12 = system32.join("d3d12.dll");
    let d3d11 = system32.join("d3d11.dll");

    if d3d12.is_file() {
        return (true, format!("Direct3D 12 runtime detected at {}.", d3d12.display()));
    }
    if d3d11.is_file() {
        return (true, format!("Direct3D 11 runtime detected at {}.", d3d11.display()));
    }

    (false, "Direct3D runtime DLLs were not detected.".into())
}

fn detect_free_disk_mb() -> Option<u64> {
    if cfg!(target_os = "linux") {
        let cwd = env::current_dir().ok()?;
        let output = Command::new("df")
            .args(["-Pk", cwd.to_string_lossy().as_ref()])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().nth(1)?;
        let available_kb = line.split_whitespace().nth(3)?.parse::<u64>().ok()?;
        return Some(available_kb / 1024);
    }

    if cfg!(windows) {
        let exe = env::current_exe().ok()?;
        let drive = exe.to_string_lossy().chars().take(2).collect::<String>();
        let script = format!(
            "(Get-CimInstance Win32_LogicalDisk -Filter \"DeviceID='{}'\").FreeSpace",
            drive.replace('\\', "")
        );
        let output = powershell(&script)?;
        let bytes = output.trim().parse::<u64>().ok()?;
        return Some(bytes / 1024 / 1024);
    }

    None
}

fn detect_hardware_encoders() -> Vec<String> {
    let ffmpeg = media::detect_ffmpeg();
    let Some(executable) = ffmpeg.executable else {
        return Vec::new();
    };

    let Ok(output) = Command::new(executable).args(["-hide_banner", "-encoders"]).output() else {
        return Vec::new();
    };

    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .map(str::trim)
        .filter(|line| {
            ["nvenc", "_qsv", "_amf", "vaapi", "videotoolbox"]
                .iter()
                .any(|needle| line.contains(needle))
        })
        .take(100)
        .map(str::to_string)
        .collect()
}

fn powershell(script: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}
