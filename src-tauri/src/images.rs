use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io::{Read, Write}, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidImageManifest {
    pub id: String,
    pub name: String,
    pub android_version: String,
    pub api: u32,
    pub architecture: String,
    pub image_type: String,
    pub disk: String,
    pub disk_format: String,
    #[serde(default)]
    pub recommended: bool,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledImage {
    pub manifest: AndroidImageManifest,
    pub directory: String,
    pub disk_path: String,
    pub valid: bool,
    pub validation_error: Option<String>,
}

pub fn images_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("images")
}

pub fn ensure_layout(data_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(images_dir(data_dir)).map_err(|e| e.to_string())
}

pub fn list_images(data_dir: &Path) -> Result<Vec<InstalledImage>, String> {
    ensure_layout(data_dir)?;
    let mut result = Vec::new();

    for entry in fs::read_dir(images_dir(data_dir)).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let bytes = fs::read(&manifest_path).map_err(|e| e.to_string())?;
        let manifest: AndroidImageManifest = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("Invalid image manifest {}: {error}", manifest_path.display());
                continue;
            }
        };
        let disk_path = entry.path().join(&manifest.disk);
        let validation_error = validate_manifest(&manifest, &disk_path).err();
        result.push(InstalledImage {
            manifest,
            directory: entry.path().to_string_lossy().to_string(),
            disk_path: disk_path.to_string_lossy().to_string(),
            valid: validation_error.is_none(),
            validation_error,
        });
    }

    result.sort_by(|a, b| b.manifest.api.cmp(&a.manifest.api));
    Ok(result)
}

pub fn remove_image(data_dir: &Path, id: &str) -> Result<(), String> {
    validate_id(id)?;
    let target_dir = images_dir(data_dir).join(id);
    if !target_dir.exists() {
        return Err(format!("Image is not installed: {id}"));
    }
    fs::remove_dir_all(&target_dir)
        .map_err(|e| format!("Failed to remove image {id}: {e}"))
}

pub fn download_image(
    data_dir: &Path,
    manifest: AndroidImageManifest,
    url: String,
) -> Result<InstalledImage, String> {
    ensure_layout(data_dir)?;
    validate_id(&manifest.id)?;

    if !matches!(manifest.disk_format.as_str(), "qcow2" | "raw") {
        return Err("diskFormat must be qcow2 or raw".into());
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Image URL must use http:// or https://".into());
    }

    let file_name = Path::new(&manifest.disk)
        .file_name()
        .ok_or_else(|| "Manifest disk must contain a file name".to_string())?
        .to_owned();

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let temp_path = images_dir(data_dir).join(format!(".{}-{stamp}.part", manifest.id));

    let result = (|| -> Result<InstalledImage, String> {
        let mut response = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60 * 30))
            .build()
            .map_err(|e| format!("Unable to create image downloader: {e}"))?
            .get(&url)
            .send()
            .map_err(|e| format!("Image download failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Image server returned an error: {e}"))?;

        let mut output = fs::File::create(&temp_path)
            .map_err(|e| format!("Unable to create temporary image file: {e}"))?;
        let mut buffer = [0u8; 1024 * 1024];

        loop {
            let read = response.read(&mut buffer)
                .map_err(|e| format!("Image download read failed: {e}"))?;
            if read == 0 {
                break;
            }
            output.write_all(&buffer[..read])
                .map_err(|e| format!("Unable to write downloaded image: {e}"))?;
        }
        output.sync_all().map_err(|e| format!("Unable to flush downloaded image: {e}"))?;

        verify_checksum(&manifest, &temp_path)?;

        let target_dir = images_dir(data_dir).join(&manifest.id);
        if target_dir.exists() {
            return Err(format!("Image id is already installed: {}", manifest.id));
        }
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

        let target_disk = target_dir.join(&file_name);
        fs::rename(&temp_path, &target_disk)
            .or_else(|_| {
                fs::copy(&temp_path, &target_disk)?;
                fs::remove_file(&temp_path)
            })
            .map_err(|e| format!("Unable to install downloaded image: {e}"))?;

        let mut stored_manifest = manifest;
        stored_manifest.disk = file_name.to_string_lossy().to_string();
        stored_manifest.source_url = Some(url.clone());
        let encoded = serde_json::to_vec_pretty(&stored_manifest).map_err(|e| e.to_string())?;
        fs::write(target_dir.join("manifest.json"), encoded).map_err(|e| e.to_string())?;

        Ok(InstalledImage {
            manifest: stored_manifest,
            directory: target_dir.to_string_lossy().to_string(),
            disk_path: target_disk.to_string_lossy().to_string(),
            valid: true,
            validation_error: None,
        })
    })();

    if result.is_err() && temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

pub fn repair_image(data_dir: &Path, id: &str) -> Result<InstalledImage, String> {
    ensure_layout(data_dir)?;
    validate_id(id)?;
    let target_dir = images_dir(data_dir).join(id);
    let manifest_path = target_dir.join("manifest.json");
    if !manifest_path.is_file() {
        return Err(format!("Image manifest is missing: {}", manifest_path.display()));
    }

    let bytes = fs::read(&manifest_path).map_err(|e| e.to_string())?;
    let manifest: AndroidImageManifest = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Invalid image manifest: {e}"))?;
    let url = manifest.source_url.clone()
        .filter(|value| value.starts_with("https://") || value.starts_with("http://"))
        .ok_or_else(|| "This image has no downloadable source URL for repair".to_string())?;

    let file_name = Path::new(&manifest.disk)
        .file_name()
        .ok_or_else(|| "Manifest disk must contain a file name".to_string())?
        .to_owned();
    let target_disk = target_dir.join(&file_name);
    let temp_path = target_dir.join(format!(".{}.repair.part", file_name.to_string_lossy()));

    let result = (|| -> Result<InstalledImage, String> {
        let mut response = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60 * 30))
            .build()
            .map_err(|e| format!("Unable to create image downloader: {e}"))?
            .get(&url)
            .send()
            .map_err(|e| format!("Image repair download failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Image server returned an error: {e}"))?;

        let mut output = fs::File::create(&temp_path)
            .map_err(|e| format!("Unable to create repair file: {e}"))?;
        let mut buffer = [0u8; 1024 * 1024];
        loop {
            let read = response.read(&mut buffer)
                .map_err(|e| format!("Image repair read failed: {e}"))?;
            if read == 0 {
                break;
            }
            output.write_all(&buffer[..read])
                .map_err(|e| format!("Unable to write repair image: {e}"))?;
        }
        output.sync_all().map_err(|e| format!("Unable to flush repair image: {e}"))?;
        verify_checksum(&manifest, &temp_path)?;

        if target_disk.exists() {
            fs::remove_file(&target_disk)
                .map_err(|e| format!("Unable to replace damaged image: {e}"))?;
        }
        fs::rename(&temp_path, &target_disk)
            .or_else(|_| {
                fs::copy(&temp_path, &target_disk)?;
                fs::remove_file(&temp_path)
            })
            .map_err(|e| format!("Unable to install repaired image: {e}"))?;

        Ok(InstalledImage {
            manifest,
            directory: target_dir.to_string_lossy().to_string(),
            disk_path: target_disk.to_string_lossy().to_string(),
            valid: true,
            validation_error: None,
        })
    })();

    if result.is_err() && temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

pub fn register_image(
    data_dir: &Path,
    manifest: AndroidImageManifest,
    source_disk: &str,
) -> Result<InstalledImage, String> {
    ensure_layout(data_dir)?;
    validate_id(&manifest.id)?;

    if !matches!(manifest.disk_format.as_str(), "qcow2" | "raw") {
        return Err("diskFormat must be qcow2 or raw".into());
    }

    let source = Path::new(source_disk);
    if !source.is_file() {
        return Err(format!("Source disk does not exist: {source_disk}"));
    }

    verify_checksum(&manifest, source)?;

    let target_dir = images_dir(data_dir).join(&manifest.id);
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let file_name = Path::new(&manifest.disk)
        .file_name()
        .ok_or_else(|| "Manifest disk must contain a file name".to_string())?
        .to_owned();
    let target_disk = target_dir.join(&file_name);

    fs::copy(source, &target_disk).map_err(|e| format!("Failed to copy image: {e}"))?;

    let mut stored_manifest = manifest;
    stored_manifest.disk = file_name.to_string_lossy().to_string();
    let encoded = serde_json::to_vec_pretty(&stored_manifest).map_err(|e| e.to_string())?;
    fs::write(target_dir.join("manifest.json"), encoded).map_err(|e| e.to_string())?;

    Ok(InstalledImage {
        manifest: stored_manifest,
        directory: target_dir.to_string_lossy().to_string(),
        disk_path: target_disk.to_string_lossy().to_string(),
        valid: true,
        validation_error: None,
    })
}

fn validate_manifest(manifest: &AndroidImageManifest, disk_path: &Path) -> Result<(), String> {
    validate_id(&manifest.id)?;
    if manifest.architecture != "x86_64" {
        return Err(format!("MVP supports x86_64 images; manifest is {}", manifest.architecture));
    }
    if !matches!(manifest.disk_format.as_str(), "qcow2" | "raw") {
        return Err(format!("Unsupported disk format: {}", manifest.disk_format));
    }
    if !disk_path.is_file() {
        return Err(format!("Disk file is missing: {}", disk_path.display()));
    }
    verify_checksum(manifest, disk_path)?;
    Ok(())
}

fn verify_checksum(manifest: &AndroidImageManifest, path: &Path) -> Result<(), String> {
    let Some(expected) = manifest.sha256.as_deref().map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    if expected.len() != 64 || !expected.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Manifest SHA-256 must contain exactly 64 hexadecimal characters".into());
    }

    let actual = sha256_file(path)?;
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!(
            "SHA-256 mismatch for {}: expected {}, got {}",
            path.display(),
            expected,
            actual
        ));
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Unable to open {} for checksum: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];

    loop {
        let read = file.read(&mut buffer)
            .map_err(|e| format!("Unable to read {} for checksum: {e}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("Image id may contain only letters, numbers, '-' and '_'".into());
    }
    Ok(())
}
