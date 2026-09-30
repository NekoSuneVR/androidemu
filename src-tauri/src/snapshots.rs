use crate::runtime;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: u64,
}

fn instance_dir(data_dir: &Path, instance_id: &str) -> PathBuf {
    data_dir.join("instances").join(instance_id)
}

fn snapshots_dir(data_dir: &Path, instance_id: &str) -> PathBuf {
    instance_dir(data_dir, instance_id).join("snapshots")
}

fn runtime_disk(data_dir: &Path, instance_id: &str) -> PathBuf {
    instance_dir(data_dir, instance_id).join("disks").join("runtime.qcow2")
}

pub fn list(data_dir: &Path, instance_id: &str) -> Result<Vec<SnapshotInfo>, String> {
    let dir = snapshots_dir(data_dir, instance_id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut snapshots = Vec::new();

    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(entry.path()).map_err(|e| e.to_string())?;
        match serde_json::from_slice::<SnapshotInfo>(&bytes) {
            Ok(snapshot) => snapshots.push(snapshot),
            Err(error) => eprintln!("Skipping invalid snapshot metadata {}: {error}", entry.path().display()),
        }
    }

    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(snapshots)
}

pub fn create(
    data_dir: &Path,
    instance_id: &str,
    name: String,
    description: String,
) -> Result<SnapshotInfo, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Snapshot name cannot be empty".into());
    }

    let disk = runtime_disk(data_dir, instance_id);
    if !disk.is_file() {
        return Err("Instance runtime disk does not exist yet. Start the instance at least once first.".into());
    }

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let id = format!("snap-{created_at}");

    run_qemu_img(&[
        "snapshot".into(),
        "-c".into(),
        id.clone(),
        disk.to_string_lossy().to_string(),
    ])?;

    let snapshot = SnapshotInfo {
        id: id.clone(),
        name: name.to_string(),
        description: description.trim().to_string(),
        created_at,
    };

    let dir = snapshots_dir(data_dir, instance_id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let encoded = serde_json::to_vec_pretty(&snapshot).map_err(|e| e.to_string())?;
    fs::write(dir.join(format!("{id}.json")), encoded).map_err(|e| e.to_string())?;
    Ok(snapshot)
}


pub fn create_clean(data_dir:&Path, instance_id:&str) -> Result<SnapshotInfo,String> {
    create(data_dir,instance_id,"Clean snapshot".into(),"Known-clean state created by NekoDroid.".into())
}

pub fn create_rooted(data_dir:&Path, instance_id:&str) -> Result<SnapshotInfo,String> {
    create(data_dir,instance_id,"Rooted snapshot".into(),"Root-capable state created by NekoDroid.".into())
}

pub fn cleanup(data_dir:&Path, instance_id:&str, keep:usize) -> Result<usize,String> {
    let keep=keep.clamp(1,100);
    let all=list(data_dir,instance_id)?;
    let mut removed=0;
    for snap in all.into_iter().skip(keep) {
        delete(data_dir,instance_id,&snap.id)?;
        removed+=1;
    }
    Ok(removed)
}

pub fn auto_snapshot(data_dir:&Path, instance_id:&str, reason:&str) -> Result<Option<SnapshotInfo>,String> {
    if !runtime_disk(data_dir,instance_id).is_file(){return Ok(None);}
    let label=match reason {
        "pre-root"=>"Before root",
        "pre-update"=>"Before Android update",
        _=>"Automatic snapshot",
    };
    create(data_dir,instance_id,label.into(),format!("Automatic safety snapshot: {reason}")).map(Some)
}

pub fn rename(
    data_dir: &Path,
    instance_id: &str,
    snapshot_id: &str,
    name: String,
    description: String,
) -> Result<SnapshotInfo, String> {
    validate_snapshot_id(snapshot_id)?;
    let name = name.trim();
    if name.is_empty() {
        return Err("Snapshot name cannot be empty".into());
    }

    let path = snapshots_dir(data_dir, instance_id).join(format!("{snapshot_id}.json"));
    let bytes = fs::read(&path).map_err(|e| format!("Unable to read snapshot metadata: {e}"))?;
    let mut snapshot: SnapshotInfo = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Invalid snapshot metadata: {e}"))?;

    snapshot.name = name.to_string();
    snapshot.description = description.trim().to_string();
    let encoded = serde_json::to_vec_pretty(&snapshot).map_err(|e| e.to_string())?;
    fs::write(path, encoded).map_err(|e| e.to_string())?;
    Ok(snapshot)
}

pub fn restore(data_dir: &Path, instance_id: &str, snapshot_id: &str) -> Result<(), String> {
    validate_snapshot_id(snapshot_id)?;
    let disk = runtime_disk(data_dir, instance_id);
    if !disk.is_file() {
        return Err("Instance runtime disk does not exist".into());
    }
    run_qemu_img(&[
        "snapshot".into(),
        "-a".into(),
        snapshot_id.into(),
        disk.to_string_lossy().to_string(),
    ])?;
    Ok(())
}

pub fn delete(data_dir: &Path, instance_id: &str, snapshot_id: &str) -> Result<(), String> {
    validate_snapshot_id(snapshot_id)?;
    let disk = runtime_disk(data_dir, instance_id);
    if !disk.is_file() {
        return Err("Instance runtime disk does not exist".into());
    }

    run_qemu_img(&[
        "snapshot".into(),
        "-d".into(),
        snapshot_id.into(),
        disk.to_string_lossy().to_string(),
    ])?;

    let metadata = snapshots_dir(data_dir, instance_id).join(format!("{snapshot_id}.json"));
    if metadata.exists() {
        fs::remove_file(metadata).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn validate_snapshot_id(id: &str) -> Result<(), String> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("Invalid snapshot ID".into());
    }
    Ok(())
}

fn run_qemu_img(args: &[String]) -> Result<(), String> {
    let executable = runtime::find_runtime_tool("qemu-img")
        .ok_or_else(|| if cfg!(windows) {
            "MSYS2 qemu-img was not found; install the MSYS2 QEMU runtime or set MSYS2_ROOT".to_string()
        } else {
            "qemu-img was not found in PATH".to_string()
        })?;
    let mut command = Command::new(&executable);
    runtime::apply_runtime_environment(&mut command, &executable);
    let output = command
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run qemu-img: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "qemu-img snapshot command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

