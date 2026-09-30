use crate::models::{AndroidInstance, CreateInstanceRequest, UpdateInstanceRequest};
use std::{
    fs,
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn instances_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("instances")
}

pub fn ensure_layout(data_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(instances_dir(data_dir))
}

pub fn instance_dir(data_dir: &Path, id: &str) -> PathBuf {
    instances_dir(data_dir).join(id)
}

pub fn config_path(data_dir: &Path, id: &str) -> PathBuf {
    instance_dir(data_dir, id).join("config.json")
}

pub fn load_instances(data_dir: &Path) -> Result<Vec<AndroidInstance>, String> {
    ensure_layout(data_dir).map_err(|e| e.to_string())?;
    let mut result = Vec::new();

    for entry in fs::read_dir(instances_dir(data_dir)).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }

        let config = entry.path().join("config.json");
        if !config.exists() {
            continue;
        }

        let bytes = fs::read(&config).map_err(|e| format!("{}: {e}", config.display()))?;
        match serde_json::from_slice::<AndroidInstance>(&bytes) {
            Ok(instance) => result.push(instance),
            Err(e) => eprintln!("Skipping invalid instance {}: {e}", config.display()),
        }
    }

    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(result)
}

pub fn load_instance(data_dir: &Path, id: &str) -> Result<AndroidInstance, String> {
    let path = config_path(data_dir, id);
    let bytes = fs::read(&path).map_err(|e| format!("Unable to read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Invalid instance config: {e}"))
}

pub fn save_instance(data_dir: &Path, instance: &AndroidInstance) -> Result<(), String> {
    let dir = instance_dir(data_dir, &instance.id);
    fs::create_dir_all(dir.join("logs")).map_err(|e| e.to_string())?;
    fs::create_dir_all(dir.join("snapshots")).map_err(|e| e.to_string())?;

    let encoded = serde_json::to_vec_pretty(instance).map_err(|e| e.to_string())?;
    fs::write(config_path(data_dir, &instance.id), encoded).map_err(|e| e.to_string())
}

pub fn create_instance(data_dir: &Path, request: CreateInstanceRequest) -> Result<AndroidInstance, String> {
    validate_request(&request)?;

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();

    let instance = AndroidInstance {
        id: format!("instance-{millis}"),
        name: request.name.trim().to_string(),
        android_version: request.android_version,
        profile: request.profile,
        status: "stopped".into(),
        cpu_cores: request.cpu_cores,
        ram_mb: request.ram_mb,
        adb_port: request.adb_port,
        adb_enabled: request.adb_enabled,
        headless: request.headless,
        root_mode: request.root_mode,
        image_path: request.image_path.filter(|p| !p.trim().is_empty()),
        process_id: None,
    };

    save_instance(data_dir, &instance)?;
    Ok(instance)
}

pub fn update_instance(
    data_dir: &Path,
    id: &str,
    request: UpdateInstanceRequest,
) -> Result<AndroidInstance, String> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err("Instance name cannot be empty".into());
    }

    let mut instance = load_instance(data_dir, id)?;
    if instance.status == "running" {
        return Err("Stop the instance before changing its name or ADB setting".into());
    }

    instance.name = name.to_string();
    instance.adb_enabled = request.adb_enabled;
    instance.headless = request.headless;
    save_instance(data_dir, &instance)?;
    Ok(instance)
}

pub fn clone_instance(data_dir: &Path, id: &str, name: String) -> Result<AndroidInstance, String> {
    let source = load_instance(data_dir, id)?;
    let name = name.trim();
    if name.is_empty() {
        return Err("Clone name cannot be empty".into());
    }

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();

    let existing = load_instances(data_dir)?;
    let next_adb_port = existing
        .iter()
        .map(|item| item.adb_port)
        .max()
        .unwrap_or(5554)
        .saturating_add(1);

    let mut clone = source.clone();
    clone.id = format!("instance-{millis}");
    clone.name = name.to_string();
    clone.status = "stopped".into();
    clone.process_id = None;
    clone.adb_port = next_adb_port;
    clone.adb_enabled = false;

    save_instance(data_dir, &clone)?;

    let source_disk = instance_dir(data_dir, id).join("disks").join("runtime.qcow2");
    if source_disk.is_file() {
        let target_disks = instance_dir(data_dir, &clone.id).join("disks");
        fs::create_dir_all(&target_disks).map_err(|e| e.to_string())?;
        fs::copy(&source_disk, target_disks.join("runtime.qcow2"))
            .map_err(|e| format!("Failed to copy cloned runtime disk: {e}"))?;
    }

    Ok(clone)
}

pub fn delete_instance(data_dir: &Path, id: &str) -> Result<(), String> {
    let dir = instance_dir(data_dir, id);
    if !dir.exists() {
        return Err("Instance does not exist".into());
    }
    fs::remove_dir_all(dir).map_err(|e| e.to_string())
}

fn validate_request(request: &CreateInstanceRequest) -> Result<(), String> {
    if request.name.trim().is_empty() {
        return Err("Instance name cannot be empty".into());
    }
    if !(1..=64).contains(&request.cpu_cores) {
        return Err("CPU cores must be between 1 and 64".into());
    }
    if !(512..=131_072).contains(&request.ram_mb) {
        return Err("RAM must be between 512 MB and 128 GB".into());
    }
    if request.adb_port == 0 {
        return Err("ADB port must be between 1 and 65535".into());
    }
    Ok(())
}
