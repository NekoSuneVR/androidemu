use crate::{
    models::{AndroidInstance, HostCapabilities, QemuInfo, RuntimeActionResult, RuntimeLogs},
    storage,
    performance,
    graphics,
    passthrough,
    adb,
};
use base64::Engine as _;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Clone)]
pub struct RuntimeState {
    pub data_dir: PathBuf,
    pub processes: Arc<Mutex<HashMap<String, Child>>>,
}

impl RuntimeState {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        storage::ensure_layout(&data_dir).map_err(|e| e.to_string())?;
        Ok(Self {
            data_dir,
            processes: Arc::new(Mutex::new(HashMap::new())),
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
            let msys2_path = qemu.executable.as_deref().map(|p| p.to_ascii_lowercase().contains("\\msys64\\")).unwrap_or(false);
            (
                "MSYS2 + WHPX".to_string(),
                whpx,
                if !qemu.found {
                    "MSYS2 runtime was not found automatically. Install MSYS2 with the UCRT64/MINGW64 QEMU package, or set MSYS2_ROOT only for a custom install location.".to_string()
                } else if whpx && msys2_path {
                    "MSYS2 Android VM runtime detected with Windows Hypervisor Platform acceleration.".to_string()
                } else if whpx {
                    "Windows VM runtime detected with Windows Hypervisor Platform acceleration.".to_string()
                } else {
                    "MSYS2 runtime was found, but WHPX is unavailable. Enable Windows Hypervisor Platform in Windows Features.".to_string()
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

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() || paths.iter().any(|existing| existing.eq_ignore_ascii_case(&path)) {
        return;
    }
    paths.push(path);
}

fn windows_msys2_roots() -> Vec<PathBuf> {
    let mut roots = Vec::<PathBuf>::new();

    for key in ["MSYS2_ROOT", "MSYS2_PATH"] {
        if let Some(value) = env::var_os(key) {
            push_unique_path(&mut roots, PathBuf::from(value));
        }
    }

    for path in [
        PathBuf::from(r"C:\msys64"),
        PathBuf::from(r"C:\msys32"),
    ] {
        push_unique_path(&mut roots, path);
    }

    for key in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(base) = env::var_os(key) {
            let base = PathBuf::from(base);
            push_unique_path(&mut roots, base.join("MSYS2"));
            push_unique_path(&mut roots, base.join("msys64"));
            push_unique_path(&mut roots, base.join("Programs").join("MSYS2"));
        }
    }

    // Pick up Scoop/Chocolatey/custom installs when pacman or bash is already reachable.
    for probe in ["pacman.exe", "bash.exe"] {
        if let Some(found) = find_in_path(probe) {
            let mut current = found.parent().map(Path::to_path_buf);
            while let Some(dir) = current {
                if dir.join("usr").join("bin").join("pacman.exe").is_file()
                    || dir.join("ucrt64").join("bin").is_dir()
                    || dir.join("mingw64").join("bin").is_dir()
                {
                    push_unique_path(&mut roots, dir.clone());
                    break;
                }
                current = dir.parent().map(Path::to_path_buf);
            }
        }
    }

    roots.into_iter().filter(|root| root.is_dir()).collect()
}

pub(crate) fn find_runtime_tool(name: &str) -> Option<PathBuf> {
    if !cfg!(windows) {
        return find_in_path(name);
    }

    let exe_name = if name.to_ascii_lowercase().ends_with(".exe") {
        name.to_string()
    } else {
        format!("{name}.exe")
    };

    for root in windows_msys2_roots() {
        for prefix in ["ucrt64", "mingw64", "clang64", "clangarm64", "mingw32", "usr"] {
            let candidate = root.join(prefix).join("bin").join(&exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    find_in_path(&exe_name).or_else(|| find_in_path(name))
}

pub(crate) fn apply_runtime_environment(command: &mut Command, tool_path: &Path) {
    if !cfg!(windows) {
        return;
    }

    let Some(bin_dir) = tool_path.parent() else { return; };
    let Some(prefix_dir) = bin_dir.parent() else { return; };
    let Some(root) = prefix_dir.parent() else { return; };

    let mut path_parts = Vec::<PathBuf>::new();
    push_unique_path(&mut path_parts, bin_dir.to_path_buf());
    push_unique_path(&mut path_parts, root.join("usr").join("bin"));
    push_unique_path(&mut path_parts, root.join("bin"));

    if let Some(existing) = env::var_os("PATH") {
        path_parts.extend(env::split_paths(&existing));
    }

    if let Ok(joined) = env::join_paths(path_parts) {
        command.env("PATH", joined);
    }
    command.env("MSYS2_ROOT", root);
    if let Some(prefix) = prefix_dir.file_name().and_then(|v| v.to_str()) {
        command.env("MSYSTEM", prefix.to_ascii_uppercase());
    }
}

pub fn detect_qemu() -> QemuInfo {
    let executable = if cfg!(windows) {
        find_runtime_tool("qemu-system-x86_64")
    } else {
        find_in_path("qemu-system-x86_64")
    };

    let Some(path) = executable else {
        return QemuInfo {
            found: false,
            executable: None,
            version: None,
            accelerators: Vec::new(),
        };
    };

    let version = {
        let mut command = Command::new(&path);
        apply_runtime_environment(&mut command, &path);
        command.arg("--version").output()
    }
        .ok()
        .and_then(|o| {
            let text = String::from_utf8_lossy(&o.stdout);
            text.lines().next().map(str::to_string)
        });

    let accelerators = {
        let mut command = Command::new(&path);
        apply_runtime_environment(&mut command, &path);
        command.args(["-accel", "help"]).output()
    }
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


pub fn set_root_on_next_boot(state:&RuntimeState,id:&str,enabled:bool)->Result<(),String>{
    let dir=storage::instance_dir(&state.data_dir,id);
    fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let marker=dir.join("root-on-next-boot");
    if enabled{fs::write(marker,b"1").map_err(|e|e.to_string())?;}else if marker.exists(){fs::remove_file(marker).map_err(|e|e.to_string())?;}
    Ok(())
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
        .ok_or_else(|| if cfg!(windows) {
            "MSYS2 runtime was not found automatically. Install MSYS2 and its UCRT64/MINGW64 QEMU runtime package.".to_string()
        } else {
            "QEMU qemu-system-x86_64 was not found in PATH".to_string()
        })?;

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

    let base_disk_format = if image_path.to_ascii_lowercase().ends_with(".qcow2") {
        "qcow2"
    } else {
        "raw"
    };
    let runtime_disk = ensure_runtime_overlay(state, id, &image_path, base_disk_format)?;
    let cpu_model = if accelerator == "kvm" { "host" } else { "max" };

    let performance = performance::load(&state.data_dir).unwrap_or_default();
    let graphics = graphics::load(&state.data_dir).unwrap_or_default();
    let mut qemu_args = build_qemu_args(&instance, &runtime_disk, "qcow2", accelerator, cpu_model);
    qemu_args.extend(crate::images::boot_component_args(&state.data_dir,&image_path)?);
    let (video_device, use_gl)=graphics::qemu_video_device(&graphics);
    if let Some(index)=qemu_args.iter().position(|arg|arg=="virtio-vga"){qemu_args[index]=video_device.into();}
    if use_gl && !instance.headless {
        qemu_args.push("-display".into());
        qemu_args.push("gtk,gl=on".into());
    }
    if let Some(index) = qemu_args.iter().position(|arg| arg == "-drive").and_then(|i| qemu_args.get(i + 1).map(|_| i + 1)) {
        qemu_args[index] = format!("file={runtime_disk},if=virtio,format=qcow2,{}", performance::qemu_drive_options(&performance));
    }
    if performance.huge_pages && cfg!(target_os = "linux") && Path::new("/dev/hugepages").exists() {
        qemu_args.extend([
            "-object".into(),
            format!("memory-backend-file,id=nekoram,size={}M,mem-path=/dev/hugepages,share=on", instance.ram_mb),
            "-numa".into(),
            "node,memdev=nekoram".into(),
        ]);
    }
    let passthrough_settings=passthrough::load(&state.data_dir).unwrap_or_default();
    qemu_args.extend(passthrough::qemu_args(&passthrough_settings));

    if let Some(audio_backend) = detect_virtio_audio_backend(&qemu_path) {
        qemu_args.push("-audiodev".into());
        qemu_args.push(format!("{audio_backend},id=nekodroid_audio"));
        qemu_args.push("-device".into());
        qemu_args.push("virtio-sound-pci,audiodev=nekodroid_audio".into());
    }

    let mut command = Command::new(&qemu_path);
    apply_runtime_environment(&mut command, Path::new(&qemu_path));
    command
        .args(qemu_args)
        .envs(performance::launch_env(&performance))
        .envs(graphics::launch_env(&graphics))
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));

    let child = command.spawn().map_err(|e| format!("Failed to start QEMU: {e}"))?;
    let pid = child.id();
    if cfg!(target_os = "linux") && !performance.cpu_affinity.trim().is_empty() {
        let _ = Command::new("taskset")
            .args(["-pc", performance.cpu_affinity.trim(), &pid.to_string()])
            .output();
    }

    state
        .processes
        .lock()
        .map_err(|_| "Runtime process lock poisoned")?
        .insert(id.to_string(), child);

    instance.status = "running".into();
    instance.process_id = Some(pid);
    storage::save_instance(&state.data_dir, &instance)?;

    let root_marker=storage::instance_dir(&state.data_dir,id).join("root-on-next-boot");
    if root_marker.exists() && instance.adb_enabled {
        let marker=root_marker.clone();
        let port=instance.adb_port;
        thread::spawn(move||{
            for _ in 0..60 {
                let _=adb::connect(port);
                if let Ok(status)=adb::get_state(port) {
                    if status.success {
                        let _=adb::root(port);
                        let _=fs::remove_file(&marker);
                        break;
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
        });
    }
    if instance.adb_enabled && (performance.startup_optimization || performance.minimize_background_services) {
        let port=instance.adb_port;
        let startup=performance.startup_optimization;
        let minimize=performance.minimize_background_services;
        thread::spawn(move||{
            for _ in 0..60 {
                let _=adb::connect(port);
                if let Ok(status)=adb::get_state(port) {
                    if status.success {
                        if startup {
                            let _=adb::shell(port,"settings put global window_animation_scale 0.5; settings put global transition_animation_scale 0.5; settings put global animator_duration_scale 0.5".into());
                        }
                        if minimize {
                            let _=adb::shell(port,"settings put global activity_manager_constants max_cached_processes=16; cmd deviceidle enable 2>/dev/null || true".into());
                        }
                        if performance.ram_compression {
                            let _=adb::root_shell(port,"swapoff /dev/block/zram0 2>/dev/null || true; echo 1 > /sys/block/zram0/reset 2>/dev/null || true; echo lz4 > /sys/block/zram0/comp_algorithm 2>/dev/null || true; echo 1073741824 > /sys/block/zram0/disksize 2>/dev/null || true; mkswap /dev/block/zram0 2>/dev/null || true; swapon /dev/block/zram0 2>/dev/null || true; cat /proc/swaps".into());
                        }
                        break;
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
        });
    }

    Ok(RuntimeActionResult {
        instance_id: id.to_string(),
        status: "running".into(),
        message: format!("QEMU started with PID {pid}"),
        process_id: Some(pid),
    })
}

pub fn pause_instance(state: &RuntimeState, id: &str) -> Result<RuntimeActionResult, String> {
    refresh_processes(state)?;
    let pid = {
        let processes = state.processes.lock().map_err(|_| "Runtime process lock poisoned")?;
        processes.get(id).map(|child| child.id())
    }.ok_or_else(|| "Instance is not running in this NekoDroid session".to_string())?;

    let mut instance = storage::load_instance(&state.data_dir, id)?;
    qmp_execute(qmp_port(&instance), "stop")?;
    instance.status = "paused".into();
    instance.process_id = Some(pid);
    storage::save_instance(&state.data_dir, &instance)?;

    Ok(RuntimeActionResult {
        instance_id: id.to_string(),
        status: "paused".into(),
        message: "QEMU virtual CPUs paused through QMP".into(),
        process_id: Some(pid),
    })
}

pub fn resume_instance(state: &RuntimeState, id: &str) -> Result<RuntimeActionResult, String> {
    refresh_processes(state)?;
    let pid = {
        let processes = state.processes.lock().map_err(|_| "Runtime process lock poisoned")?;
        processes.get(id).map(|child| child.id())
    }.ok_or_else(|| "Instance is not running in this NekoDroid session".to_string())?;

    let mut instance = storage::load_instance(&state.data_dir, id)?;
    qmp_execute(qmp_port(&instance), "cont")?;
    instance.status = "running".into();
    instance.process_id = Some(pid);
    storage::save_instance(&state.data_dir, &instance)?;

    Ok(RuntimeActionResult {
        instance_id: id.to_string(),
        status: "running".into(),
        message: "QEMU virtual CPUs resumed through QMP".into(),
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
                Ok(Some(status)) => ended.push((id.clone(), Some(status))),
                Ok(None) => {}
                Err(_) => ended.push((id.clone(), None)),
            }
        }
        for (id, _) in &ended {
            processes.remove(id);
        }
    }

    for (id, exit_status) in ended {
        if let Ok(mut instance) = storage::load_instance(&state.data_dir, &id) {
            instance.status = "stopped".into();
            instance.process_id = None;
            let _ = storage::save_instance(&state.data_dir, &instance);
        }

        if exit_status.as_ref().map(|status| !status.success()).unwrap_or(true) {
            let logs_dir = storage::instance_dir(&state.data_dir, &id).join("logs");
            let _ = fs::create_dir_all(&logs_dir);
            let status_text = exit_status
                .map(|status| status.to_string())
                .unwrap_or_else(|| "unknown process error".into());
            let report = format!(
                "NekoDroid detected an unexpected QEMU exit.\nInstance: {id}\nExit status: {status_text}\n"
            );
            let _ = fs::write(logs_dir.join("crash-report.log"), report);
        }
    }
    Ok(())
}


pub fn capture_framebuffer(state:&RuntimeState,id:&str,destination:String)->Result<String,String>{
    refresh_processes(state)?;
    if !state.processes.lock().map_err(|_|"Runtime process lock poisoned")?.contains_key(id){return Err("Instance must be running for framebuffer capture".into());}
    if destination.trim().is_empty(){return Err("Destination path is required".into());}
    let instance=storage::load_instance(&state.data_dir,id)?;
    let path=PathBuf::from(&destination);
    if let Some(parent)=path.parent(){if !parent.as_os_str().is_empty(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}}
    let requested_format=if path.extension().and_then(|v|v.to_str()).map(|v|v.eq_ignore_ascii_case("png")).unwrap_or(false){"png"}else{"ppm"};
    let args=serde_json::json!({"filename":destination,"format":requested_format});
    if let Err(first)=qmp_execute_with_arguments(qmp_port(&instance),"screendump",Some(args)){
        let fallback=serde_json::json!({"filename":path.to_string_lossy().to_string()});
        qmp_execute_with_arguments(qmp_port(&instance),"screendump",Some(fallback)).map_err(|second|format!("{first}; fallback failed: {second}"))?;
    }
    Ok(path.to_string_lossy().to_string())
}



pub fn capture_framebuffer_base64(state:&RuntimeState,id:&str)->Result<String,String>{
    let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
    let temp=env::temp_dir().join(format!("nekodroid-webrtc-{stamp}.ppm"));
    capture_framebuffer(state,id,temp.to_string_lossy().to_string())?;
    let bytes=fs::read(&temp).map_err(|e|e.to_string())?;
    let _=fs::remove_file(&temp);
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

pub fn record_framebuffer(state:&RuntimeState,id:&str,destination:String,seconds:u32,fps:u32)->Result<String,String>{
    let seconds=seconds.clamp(1,120);
    let fps=fps.clamp(1,60);
    refresh_processes(state)?;
    if !state.processes.lock().map_err(|_|"Runtime process lock poisoned")?.contains_key(id){return Err("Instance must be running for framebuffer recording".into());}
    let ffmpeg=find_in_path(if cfg!(windows){"ffmpeg.exe"}else{"ffmpeg"}).ok_or("FFmpeg is required for framebuffer recording")?;
    let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
    let temp=env::temp_dir().join(format!("nekodroid-fb-{stamp}"));fs::create_dir_all(&temp).map_err(|e|e.to_string())?;
    let frames=seconds.saturating_mul(fps);
    let frame_delay=Duration::from_secs_f64(1.0/fps as f64);
    for index in 0..frames{
        let frame=temp.join(format!("frame-{index:06}.ppm"));
        capture_framebuffer(state,id,frame.to_string_lossy().to_string())?;
        thread::sleep(frame_delay);
    }
    let output=Command::new(ffmpeg).args([
        "-hide_banner","-y","-framerate",&fps.to_string(),"-i",
        &temp.join("frame-%06d.ppm").to_string_lossy(),
        "-c:v","libx264","-pix_fmt","yuv420p",&destination
    ]).output().map_err(|e|e.to_string())?;
    let _=fs::remove_dir_all(&temp);
    if !output.status.success(){return Err(String::from_utf8_lossy(&output.stderr).to_string());}
    Ok(destination)
}

pub fn read_logs(state: &RuntimeState, id: &str) -> Result<RuntimeLogs, String> {
    let logs_dir = storage::instance_dir(&state.data_dir, id).join("logs");
    Ok(RuntimeLogs {
        stdout: read_log_tail(&logs_dir.join("qemu.out.log"))?,
        stderr: read_log_tail(&logs_dir.join("qemu.err.log"))?,
        crash_report: read_log_tail(&logs_dir.join("crash-report.log"))?,
    })
}

fn read_log_tail(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok(String::new());
    }
    let bytes = fs::read(path).map_err(|e| format!("Unable to read {}: {e}", path.display()))?;
    let start = bytes.len().saturating_sub(200_000);
    Ok(String::from_utf8_lossy(&bytes[start..]).to_string())
}

pub fn factory_reset(state: &RuntimeState, id: &str) -> Result<RuntimeActionResult, String> {
    refresh_processes(state)?;
    if state.processes.lock().map_err(|_| "Runtime process lock poisoned")?.contains_key(id) {
        return Err("Stop the instance before factory reset".into());
    }

    let instance = storage::load_instance(&state.data_dir, id)?;
    let dir = storage::instance_dir(&state.data_dir, id);
    let runtime_disk = dir.join("disks").join("runtime.qcow2");
    let snapshots_dir = dir.join("snapshots");

    if runtime_disk.exists() {
        fs::remove_file(&runtime_disk)
            .map_err(|e| format!("Unable to remove runtime disk: {e}"))?;
    }
    if snapshots_dir.exists() {
        fs::remove_dir_all(&snapshots_dir)
            .map_err(|e| format!("Unable to clear snapshots: {e}"))?;
    }
    fs::create_dir_all(&snapshots_dir).map_err(|e| e.to_string())?;

    Ok(RuntimeActionResult {
        instance_id: id.to_string(),
        status: instance.status,
        message: "Factory reset complete. A fresh writable disk will be created from the base image on next start.".into(),
        process_id: None,
    })
}

pub fn runtime_status(state: &RuntimeState, id: &str) -> Result<AndroidInstance, String> {
    refresh_processes(state)?;
    let mut instance = storage::load_instance(&state.data_dir, id)?;
    let processes = state.processes.lock().map_err(|_| "Runtime process lock poisoned")?;
    if let Some(child) = processes.get(id) {
        if instance.status != "paused" {
            instance.status = "running".into();
        }
        instance.process_id = Some(child.id());
    } else if instance.status == "running" {
        instance.status = "stopped".into();
        instance.process_id = None;
        storage::save_instance(&state.data_dir, &instance)?;
    }
    Ok(instance)
}

fn detect_virtio_audio_backend(qemu_path: &str) -> Option<String> {
    let devices = {
        let mut command = Command::new(qemu_path);
        apply_runtime_environment(&mut command, Path::new(qemu_path));
        command.args(["-device", "help"]).output()
    }
        .ok()
        .map(|output| {
            let mut text = String::from_utf8_lossy(&output.stdout).to_string();
            text.push_str(&String::from_utf8_lossy(&output.stderr));
            text
        })?;

    if !devices.contains("virtio-sound-pci") {
        return None;
    }

    let drivers = {
        let mut command = Command::new(qemu_path);
        apply_runtime_environment(&mut command, Path::new(qemu_path));
        command.args(["-audio", "driver=help"]).output()
    }
        .ok()
        .map(|output| {
            let mut text = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            text.push_str(&String::from_utf8_lossy(&output.stderr).to_ascii_lowercase());
            text
        })?;

    let preferred: &[&str] = if cfg!(windows) {
        &["dsound", "sdl"]
    } else if cfg!(target_os = "linux") {
        &["pipewire", "pa", "alsa", "sdl"]
    } else if cfg!(target_os = "macos") {
        &["coreaudio", "sdl"]
    } else {
        &["sdl"]
    };

    preferred
        .iter()
        .find(|driver| drivers.lines().any(|line| line.trim() == **driver || line.contains(&format!(" {driver}"))))
        .map(|driver| (*driver).to_string())
}

fn ensure_runtime_overlay(
    state: &RuntimeState,
    id: &str,
    base_image: &str,
    base_format: &str,
) -> Result<String, String> {
    let disks_dir = storage::instance_dir(&state.data_dir, id).join("disks");
    fs::create_dir_all(&disks_dir).map_err(|e| e.to_string())?;
    let overlay = disks_dir.join("runtime.qcow2");

    if overlay.is_file() {
        return Ok(overlay.to_string_lossy().to_string());
    }

    let qemu_img = if cfg!(windows) {
        find_runtime_tool("qemu-img")
    } else {
        find_in_path("qemu-img")
    }
    .ok_or_else(|| if cfg!(windows) {
        "MSYS2 qemu-img was not found; install the MSYS2 QEMU runtime or set MSYS2_ROOT".to_string()
    } else {
        "qemu-img was not found in PATH; it is required to create per-instance writable disks".to_string()
    })?;

    let overlay_path = overlay.to_string_lossy().to_string();
    let mut command = Command::new(&qemu_img);
    apply_runtime_environment(&mut command, &qemu_img);
    let output = command
        .args([
            "create",
            "-f",
            "qcow2",
            "-F",
            base_format,
            "-b",
            base_image,
            overlay_path.as_str(),
        ])
        .output()
        .map_err(|e| format!("Failed to execute qemu-img: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "qemu-img failed to create runtime overlay: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(overlay_path)
}

fn qmp_port(instance: &AndroidInstance) -> u16 {
    20_000 + (instance.adb_port % 20_000)
}

fn qmp_execute_with_arguments(port:u16,command:&str,arguments:Option<serde_json::Value>)->Result<(),String>{
    let address=format!("127.0.0.1:{port}");
    let mut stream=None;
    for _ in 0..20{
        match TcpStream::connect(&address){Ok(candidate)=>{stream=Some(candidate);break;},Err(_)=>thread::sleep(Duration::from_millis(100))}
    }
    let mut stream=stream.ok_or_else(||format!("Unable to connect to QMP at {address}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).map_err(|e|e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(2))).map_err(|e|e.to_string())?;
    let reader_stream=stream.try_clone().map_err(|e|e.to_string())?;
    let mut reader=BufReader::new(reader_stream);
    let mut line=String::new();reader.read_line(&mut line).map_err(|e|format!("Failed to read QMP greeting: {e}"))?;
    if !line.contains("\"QMP\""){return Err("Invalid QMP greeting".into());}
    stream.write_all(b"{\"execute\":\"qmp_capabilities\"}\r\n").map_err(|e|e.to_string())?;read_qmp_response(&mut reader)?;
    let request=if let Some(arguments)=arguments{serde_json::json!({"execute":command,"arguments":arguments})}else{serde_json::json!({"execute":command})};
    let mut encoded=serde_json::to_vec(&request).map_err(|e|e.to_string())?;encoded.extend_from_slice(b"\r\n");
    stream.write_all(&encoded).map_err(|e|format!("Failed to send QMP command: {e}"))?;
    read_qmp_response(&mut reader)
}

fn qmp_execute(port: u16, command: &str) -> Result<(), String> {
    let address = format!("127.0.0.1:{port}");
    let mut stream = None;

    for _ in 0..20 {
        match TcpStream::connect(&address) {
            Ok(candidate) => {
                stream = Some(candidate);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(100)),
        }
    }

    let mut stream = stream.ok_or_else(|| format!("Unable to connect to QMP at {address}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(2))).map_err(|e| e.to_string())?;

    let reader_stream = stream.try_clone().map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| format!("Failed to read QMP greeting: {e}"))?;
    if !line.contains("\"QMP\"") {
        return Err("Invalid QMP greeting".into());
    }

    stream.write_all(b"{\"execute\":\"qmp_capabilities\"}\r\n")
        .map_err(|e| format!("Failed to enable QMP capabilities: {e}"))?;
    read_qmp_response(&mut reader)?;

    let request = format!("{{\"execute\":\"{command}\"}}\r\n");
    stream.write_all(request.as_bytes())
        .map_err(|e| format!("Failed to send QMP command: {e}"))?;
    read_qmp_response(&mut reader)?;
    Ok(())
}

fn read_qmp_response(reader: &mut BufReader<TcpStream>) -> Result<(), String> {
    for _ in 0..20 {
        let mut line = String::new();
        let count = reader.read_line(&mut line).map_err(|e| format!("Failed to read QMP response: {e}"))?;
        if count == 0 {
            return Err("QMP connection closed unexpectedly".into());
        }
        if line.contains("\"error\"") {
            return Err(format!("QMP returned an error: {}", line.trim()));
        }
        if line.contains("\"return\"") {
            return Ok(());
        }
    }
    Err("Timed out waiting for QMP response".into())
}

pub fn build_qemu_args(
    instance: &AndroidInstance,
    image_path: &str,
    disk_format: &str,
    accelerator: &str,
    cpu_model: &str,
) -> Vec<String> {
    let mut args = vec![
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
    ];

    args.push("-netdev".into());
    if instance.adb_enabled {
        args.push(format!(
            "user,id=net0,hostfwd=tcp:127.0.0.1:{}-:5555",
            instance.adb_port
        ));
    } else {
        args.push("user,id=net0".into());
    }
    args.push("-device".into());
    args.push("virtio-net-pci,netdev=net0".into());

    args.push("-qmp".into());
    args.push(format!(
        "tcp:127.0.0.1:{},server=on,wait=off",
        qmp_port(instance)
    ));

    if instance.headless {
        args.push("-display".into());
        args.push("none".into());
    }

    args
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
            adb_enabled: false,
            headless: false,
            root_mode: "standard".into(),
            image_path: Some("/tmp/android16.qcow2".into()),
            process_id: None,
        }
    }

    #[test]
    fn qemu_args_keep_adb_on_loopback() {
        let mut configured = instance();
        configured.adb_enabled = true;
        let args = build_qemu_args(
            &configured,
            "/tmp/android16.qcow2",
            "qcow2",
            "kvm",
            "host",
        );
        assert!(args.iter().any(|arg| arg == "user,id=net0,hostfwd=tcp:127.0.0.1:5557-:5555"));
        assert!(!args.iter().any(|arg| arg.contains("0.0.0.0")));
    }

    #[test]
    fn qemu_args_disable_adb_forwarding_by_default() {
        let args = build_qemu_args(
            &instance(),
            "/tmp/android16.qcow2",
            "qcow2",
            "kvm",
            "host",
        );
        assert!(args.iter().any(|arg| arg == "user,id=net0"));
        assert!(!args.iter().any(|arg| arg.contains("hostfwd=")));
    }

    #[test]
    fn qemu_args_include_localhost_qmp() {
        let args = build_qemu_args(
            &instance(),
            "/tmp/runtime.qcow2",
            "qcow2",
            "kvm",
            "host",
        );
        assert!(args.iter().any(|arg| arg == "tcp:127.0.0.1:25557,server=on,wait=off"));
        assert!(!args.iter().any(|arg| arg.contains("0.0.0.0")));
    }

    #[test]
    fn qemu_args_enable_headless_display() {
        let mut configured = instance();
        configured.headless = true;
        let args = build_qemu_args(
            &configured,
            "/tmp/runtime.qcow2",
            "qcow2",
            "kvm",
            "host",
        );
        assert!(args.windows(2).any(|pair| pair[0] == "-display" && pair[1] == "none"));
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
        assert!(args.windows(2).any(|pair| pair[0] == "-smp" && pair[1] == "8"));
        assert!(args.windows(2).any(|pair| pair[0] == "-m" && pair[1] == "8192"));
        assert!(args.windows(2).any(|pair| pair[0] == "-cpu" && pair[1] == "host"));
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}
