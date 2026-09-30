use crate::{
    models::{AndroidInstance, HostCapabilities, QemuInfo, RuntimeActionResult},
    storage,
};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
};

pub struct RuntimeState {
    pub data_dir: PathBuf,
    pub processes: Mutex<HashMap<String, Child>>,
}

impl RuntimeState {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        storage::ensure_layout(&data_dir).map_err(|e| e.to_string())?;
        Ok(Self {
            data_dir,
            processes: Mutex::new(HashMap::new()),
        })
    }
}

pub fn detect_host() -> HostCapabilities {
    let os = env::consts::OS.to_string();
    let arch = env::consts::ARCH.to_string();
    let qemu = detect_qemu();

    let (accelerator, available, note) = match env::consts::OS {
        "linux" => {
            let device_ok = Path::new("/dev/kvm").exists();
            let qemu_ok = qemu.accelerators.iter().any(|a| a == "kvm");
            (
                "KVM".to_string(),
                device_ok && qemu_ok,
                if !device_ok {
                    "/dev/kvm is unavailable. Enable CPU virtualization and KVM on the host.".to_string()
                } else if !qemu_ok {
                    "QEMU was found but does not advertise the KVM accelerator.".to_string()
                } else {
                    "KVM is available.".to_string()
                },
            )
        }
        "windows" => {
            let whpx = qemu.accelerators.iter().any(|a| a == "whpx");
            (
                "WHPX".to_string(),
                whpx,
                if whpx {
                    "QEMU advertises Windows Hypervisor Platform acceleration.".to_string()
                } else {
                    "WHPX is not advertised by the detected QEMU build. Enable Windows Hypervisor Platform and use a compatible QEMU build.".to_string()
                },
            )
        }
        other => (
            "TCG".to_string(),
            qemu.accelerators.iter().any(|a| a == "tcg"),
            format!("{other} currently uses QEMU software acceleration fallback."),
        ),
    };

    HostCapabilities {
        os,
        arch,
        accelerator,
        accelerator_available: available,
        virtualization_note: note,
        qemu,
    }
}

pub fn detect_qemu() -> QemuInfo {
    let candidates: &[&str] = if cfg!(windows) {
        &["qemu-system-x86_64.exe", "qemu-system-x86_64"]
    } else {
        &["qemu-system-x86_64"]
    };

    let executable = candidates.iter().find_map(|name| find_in_path(name));

    let Some(path) = executable else {
        return QemuInfo {
            found: false,
            executable: None,
            version: None,
            accelerators: Vec::new(),
        };
    };

    let version = Command::new(&path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            let text = String::from_utf8_lossy(&o.stdout);
            text.lines().next().map(str::to_string)
        });

    let accelerators = Command::new(&path)
        .args(["-accel", "help"])
        .output()
        .ok()
        .map(|o| {
            let mut all = String::from_utf8_lossy(&o.stdout).to_string();
            all.push_str(&String::from_utf8_lossy(&o.stderr));
            all.lines()
                .map(str::trim)
                .filter(|line| matches!(*line, "kvm" | "whpx" | "tcg" | "hvf" | "xen" | "hax"))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    QemuInfo {
        found: true,
        executable: Some(path.to_string_lossy().to_string()),
        version,
        accelerators,
    }
}

pub fn start_instance(state: &RuntimeState, id: &str) -> Result<RuntimeActionResult, String> {
    refresh_processes(state)?;
    {
        let processes = state.processes.lock().map_err(|_| "Runtime process lock poisoned")?;
        if let Some(child) = processes.get(id) {
            return Ok(RuntimeActionResult {
                instance_id: id.to_string(),
                status: "running".into(),
                message: "Instance is already running".into(),
                process_id: Some(child.id()),
            });
        }
    }

    let mut instance = storage::load_instance(&state.data_dir, id)?;
    let host = detect_host();
    let qemu_path = host
        .qemu
        .executable
        .ok_or_else(|| "QEMU qemu-system-x86_64 was not found in PATH".to_string())?;

    if !host.accelerator_available {
        return Err(host.virtualization_note);
    }

    let image_path = instance
        .image_path
        .clone()
        .ok_or_else(|| "This instance has no boot image configured yet".to_string())?;

    if !Path::new(&image_path).exists() {
        return Err(format!("Configured image does not exist: {image_path}"));
    }

    let dir = storage::instance_dir(&state.data_dir, id);
    fs::create_dir_all(dir.join("logs")).map_err(|e| e.to_string())?;
    let stdout = File::create(dir.join("logs").join("qemu.out.log")).map_err(|e| e.to_string())?;
    let stderr = File::create(dir.join("logs").join("qemu.err.log")).map_err(|e| e.to_string())?;

    let accelerator = if cfg!(target_os = "windows") {
        "whpx"
    } else if cfg!(target_os = "linux") {
        "kvm"
    } else {
        "tcg"
    };

    let disk_format = if image_path.to_ascii_lowercase().ends_with(".qcow2") {
        "qcow2"
    } else {
        "raw"
    };
    let cpu_model = if accelerator == "kvm" { "host" } else { "max" };

    let mut command = Command::new(qemu_path);
    command
        .args(build_qemu_args(&instance, &image_path, disk_format, accelerator, cpu_model))
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));

    let child = command.spawn().map_err(|e| format!("Failed to start QEMU: {e}"))?;
    let pid = child.id();

    state
        .processes
        .lock()
        .map_err(|_| "Runtime process lock poisoned")?
        .insert(id.to_string(), child);

    instance.status = "running".into();
    instance.process_id = Some(pid);
    storage::save_instance(&state.data_dir, &instance)?;

    Ok(RuntimeActionResult {
        instance_id: id.to_string(),
        status: "running".into(),
        message: format!("QEMU started with PID {pid}"),
        process_id: Some(pid),
    })
}

pub fn stop_instance(state: &RuntimeState, id: &str) -> Result<RuntimeActionResult, String> {
    let child = state
        .processes
        .lock()
        .map_err(|_| "Runtime process lock poisoned")?
        .remove(id);

    let Some(mut child) = child else {
        let mut instance = storage::load_instance(&state.data_dir, id)?;
        instance.status = "stopped".into();
        instance.process_id = None;
        storage::save_instance(&state.data_dir, &instance)?;
        return Ok(RuntimeActionResult {
            instance_id: id.to_string(),
            status: "stopped".into(),
            message: "Instance was not running in this NekoDroid session".into(),
            process_id: None,
        });
    };

    child.kill().map_err(|e| format!("Unable to stop QEMU: {e}"))?;
    let _ = child.wait();

    let mut instance = storage::load_instance(&state.data_dir, id)?;
    instance.status = "stopped".into();
    instance.process_id = None;
    storage::save_instance(&state.data_dir, &instance)?;

    Ok(RuntimeActionResult {
        instance_id: id.to_string(),
        status: "stopped".into(),
        message: "QEMU stopped".into(),
        process_id: None,
    })
}

pub fn refresh_processes(state: &RuntimeState) -> Result<(), String> {
    let mut ended = Vec::new();
    {
        let mut processes = state.processes.lock().map_err(|_| "Runtime process lock poisoned")?;
        for (id, child) in processes.iter_mut() {
            match child.try_wait() {
                Ok(Some(_)) => ended.push(id.clone()),
                Ok(None) => {}
                Err(_) => ended.push(id.clone()),
            }
        }
        for id in &ended {
            processes.remove(id);
        }
    }

    for id in ended {
        if let Ok(mut instance) = storage::load_instance(&state.data_dir, &id) {
            instance.status = "stopped".into();
            instance.process_id = None;
            let _ = storage::save_instance(&state.data_dir, &instance);
        }
    }
    Ok(())
}

pub fn runtime_status(state: &RuntimeState, id: &str) -> Result<AndroidInstance, String> {
    refresh_processes(state)?;
    let mut instance = storage::load_instance(&state.data_dir, id)?;
    let processes = state.processes.lock().map_err(|_| "Runtime process lock poisoned")?;
    if let Some(child) = processes.get(id) {
        instance.status = "running".into();
        instance.process_id = Some(child.id());
    } else if instance.status == "running" {
        instance.status = "stopped".into();
        instance.process_id = None;
        storage::save_instance(&state.data_dir, &instance)?;
    }
    Ok(instance)
}

pub fn build_qemu_args(
    instance: &AndroidInstance,
    image_path: &str,
    disk_format: &str,
    accelerator: &str,
    cpu_model: &str,
) -> Vec<String> {
    vec![
        "-name".into(),
        format!("NekoDroid-{}", instance.name),
        "-machine".into(),
        format!("q35,accel={accelerator}"),
        "-cpu".into(),
        cpu_model.into(),
        "-smp".into(),
        instance.cpu_cores.to_string(),
        "-m".into(),
        instance.ram_mb.to_string(),
        "-drive".into(),
        format!("file={image_path},if=virtio,format={disk_format}"),
        "-device".into(),
        "virtio-vga".into(),
        "-device".into(),
        "virtio-keyboard-pci".into(),
        "-device".into(),
        "virtio-mouse-pci".into(),
        "-netdev".into(),
        format!(
            "user,id=net0,hostfwd=tcp:127.0.0.1:{}-:5555",
            instance.adb_port
        ),
        "-device".into(),
        "virtio-net-pci,netdev=net0".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::build_qemu_args;
    use crate::models::AndroidInstance;

    fn instance() -> AndroidInstance {
        AndroidInstance {
            id: "test".into(),
            name: "Gaming".into(),
            android_version: "16".into(),
            profile: "Gaming Phone".into(),
            status: "stopped".into(),
            cpu_cores: 8,
            ram_mb: 8192,
            adb_port: 5557,
            root_mode: "standard".into(),
            image_path: Some("/tmp/android16.qcow2".into()),
            process_id: None,
        }
    }

    #[test]
    fn qemu_args_keep_adb_on_loopback() {
        let args = build_qemu_args(
            &instance(),
            "/tmp/android16.qcow2",
            "qcow2",
            "kvm",
            "host",
        );
        assert!(args.iter().any(|arg| arg == "user,id=net0,hostfwd=tcp:127.0.0.1:5557-:5555"));
        assert!(!args.iter().any(|arg| arg.contains("0.0.0.0")));
    }

    #[test]
    fn qemu_args_include_instance_resources() {
        let args = build_qemu_args(
            &instance(),
            "/tmp/android16.qcow2",
            "qcow2",
            "kvm",
            "host",
        );
        assert!(args.windows(2).any(|pair| pair == ["-smp", "8"]));
        assert!(args.windows(2).any(|pair| pair == ["-m", "8192"]));
        assert!(args.windows(2).any(|pair| pair == ["-cpu", "host"]));
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}
