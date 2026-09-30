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
    #[serde(default = "default_gms_provider")]
    pub gms_provider: String,
    #[serde(default)]
    pub certification_status: String,
    #[serde(default)]
    pub play_store_package: Option<String>,
    #[serde(default)]
    pub secure_image: bool,
    #[serde(default)]
    pub verified_boot_state: String,
    #[serde(default)]
    pub security_state: String,
    #[serde(default)]
    pub missing_hardware_features: Vec<String>,
    #[serde(default)]
    pub boot_kernel: Option<String>,
    #[serde(default)]
    pub boot_initrd: Option<String>,
    #[serde(default)]
    pub vendor_disk: Option<String>,
    #[serde(default)]
    pub root_capable: bool,
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

fn default_gms_provider() -> String { "none".into() }

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
    if !matches!(manifest.gms_provider.as_str(), "none" | "google-compatible" | "microg" | "custom-gapps") {
        return Err("gmsProvider must be none, google-compatible, microg, or custom-gapps".into());
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


pub fn import_gsi(data_dir:&Path, source:String, android_version:String, architecture:String)->Result<InstalledImage,String>{
    ensure_layout(data_dir)?;
    let source_path=Path::new(&source);
    if !source_path.is_file(){return Err("GSI image does not exist".into());}
    let ext=source_path.extension().and_then(|v|v.to_str()).unwrap_or("").to_ascii_lowercase();
    if !matches!(ext.as_str(),"img"|"raw"|"qcow2"){return Err("GSI must be .img, .raw, or .qcow2".into());}
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_secs();
    let id=format!("gsi-{}-{stamp}",android_version.replace('.',"-"));
    let disk_format=if ext=="qcow2"{"qcow2"}else{"raw"};
    let disk=format!("system.{disk_format}");
    let manifest=AndroidImageManifest{
        id:id.clone(),
        name:format!("Custom GSI Android {android_version}"),
        android_version,
        api:0,
        architecture,
        image_type:"gsi".into(),
        disk,
        disk_format:disk_format.into(),
        recommended:false,
        notes:Some("Imported custom Generic System Image. Boot compatibility depends on a matching kernel/vendor/runtime configuration.".into()),
        sha256:None,
        source_url:None,
        gms_provider:"none".into(),
        certification_status:"unknown".into(),
        play_store_package:None,
        secure_image:false,
        verified_boot_state:"unknown".into(),
        security_state:"custom GSI".into(),
        missing_hardware_features:Vec::new(),
        boot_kernel:None,
        boot_initrd:None,
        vendor_disk:None,
        root_capable:false,
    };
    register_image(data_dir,manifest,&source)
}

pub fn supported_android_versions()->Vec<String>{
    ["9","10","11","12","12L","13","14","15","16"].into_iter().map(str::to_string).collect()
}



fn image_cache_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("cache").join("android-images")
}

fn resolve_default_image_url(data_dir: &Path, root: bool) -> Result<(String, Option<String>), String> {
    let settings = load_default_image_settings(data_dir)?;
    if root {
        if let Some(url) = settings.root_developer_url.filter(|value| !value.trim().is_empty()) {
            return Ok((url, settings.root_developer_sha256));
        }
        return Err("Root developer image URL is not configured.".into());
    }

    if !settings.url.trim().is_empty() {
        return Ok((settings.url, settings.sha256));
    }

    // Zero-config source: look for a NekoDroid-managed Android image asset
    // on the latest GitHub release. This keeps the desktop installer small
    // while allowing the image to be downloaded once and cached locally.
    let client = reqwest::blocking::Client::builder()
        .user_agent("NekoDroid default Android image resolver")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let release: serde_json::Value = client
        .get("https://api.github.com/repos/NekoSuneVR/androidemu/releases/latest")
        .send()
        .map_err(|e| format!("Unable to check NekoDroid releases for the default Android image: {e}"))?
        .error_for_status()
        .map_err(|e| format!("NekoDroid release lookup failed: {e}"))?
        .json()
        .map_err(|e| format!("Invalid NekoDroid release response: {e}"))?;

    let assets = release.get("assets").and_then(|value| value.as_array())
        .ok_or_else(|| "Latest NekoDroid release did not contain any assets.".to_string())?;
    let asset = assets.iter().find(|asset| {
        let name = asset.get("name").and_then(|value| value.as_str()).unwrap_or("").to_ascii_lowercase();
        name.starts_with("nekodroid-android-16-x86_64")
            && (name.ends_with(".zip") || name.ends_with(".qcow2") || name.ends_with(".img") || name.ends_with(".raw"))
    }).ok_or_else(|| {
        "No default Android 16 x86_64 image package was found on the latest NekoDroid release. Upload a NekoDroid-Android-16-x86_64.zip/.qcow2 asset or configure a custom source in Android Images.".to_string()
    })?;

    let url = asset.get("browser_download_url").and_then(|value| value.as_str())
        .ok_or_else(|| "Default Android release asset did not contain a download URL.".to_string())?;
    Ok((url.to_string(), None))
}

fn cached_package_path(data_dir: &Path, url: &str) -> PathBuf {
    let file_name = url.split('?').next().unwrap_or(url)
        .rsplit('/').next().filter(|name| !name.trim().is_empty())
        .unwrap_or("NekoDroid-Android-16-x86_64.zip");
    image_cache_dir(data_dir).join(file_name)
}

fn download_to_cache(data_dir: &Path, url: &str, expected_sha256: Option<&str>) -> Result<PathBuf, String> {
    fs::create_dir_all(image_cache_dir(data_dir)).map_err(|e| e.to_string())?;
    let target = cached_package_path(data_dir, url);

    if target.is_file() {
        if let Some(expected) = expected_sha256.filter(|value| !value.trim().is_empty()) {
            verify_file_sha256(&target, expected)?;
        }
        return Ok(target);
    }

    let temp = target.with_extension(format!(
        "{}.part",
        target.extension().and_then(|value| value.to_str()).unwrap_or("download")
    ));
    let mut response = reqwest::blocking::Client::builder()
        .user_agent("NekoDroid Android image installer")
        .timeout(std::time::Duration::from_secs(60 * 60))
        .build()
        .map_err(|e| e.to_string())?
        .get(url)
        .send()
        .map_err(|e| format!("Default Android image download failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Default Android image server returned an error: {e}"))?;

    let mut output = fs::File::create(&temp)
        .map_err(|e| format!("Unable to create cached Android package: {e}"))?;
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let count = response.read(&mut buffer).map_err(|e| format!("Android image download read failed: {e}"))?;
        if count == 0 { break; }
        output.write_all(&buffer[..count]).map_err(|e| format!("Unable to write cached Android package: {e}"))?;
    }
    output.sync_all().map_err(|e| format!("Unable to flush cached Android package: {e}"))?;

    if let Some(expected) = expected_sha256.filter(|value| !value.trim().is_empty()) {
        verify_file_sha256(&temp, expected)?;
    }

    fs::rename(&temp, &target)
        .or_else(|_| { fs::copy(&temp, &target)?; fs::remove_file(&temp) })
        .map_err(|e| format!("Unable to finalize cached Android package: {e}"))?;
    Ok(target)
}

fn verify_file_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err(format!("Android image checksum mismatch. Expected {}, got {}.", expected.trim(), actual))
    }
}

fn install_cached_image_package(data_dir: &Path, package: &Path, source_url: &str) -> Result<InstalledImage, String> {
    let ext = package.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
    if ext != "zip" {
        let mut manifest = default_manifest(false, None);
        if matches!(ext.as_str(), "img" | "raw") {
            manifest.disk = "android16.img".into();
            manifest.disk_format = "raw".into();
        } else if ext == "qcow2" {
            manifest.disk = "android16.qcow2".into();
            manifest.disk_format = "qcow2".into();
        } else {
            return Err(format!("Unsupported cached Android image package format: .{ext}"));
        }
        manifest.source_url = Some(source_url.to_string());
        return register_image(data_dir, manifest, package.to_string_lossy().as_ref());
    }

    let file = fs::File::open(package).map_err(|e| format!("Unable to open cached Android package: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid NekoDroid Android image ZIP: {e}"))?;

    let manifest_bytes = {
        let mut entry = archive.by_name("manifest.json")
            .map_err(|_| "NekoDroid Android image ZIP must contain manifest.json at its root.".to_string())?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        bytes
    };
    let mut manifest: AndroidImageManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("Invalid Android image package manifest: {e}"))?;
    validate_id(&manifest.id)?;

    if let Some(existing) = list_images(data_dir)?.into_iter().find(|image| image.manifest.id == manifest.id && image.valid) {
        return Ok(existing);
    }

    let target = images_dir(data_dir).join(&manifest.id);
    let temp = images_dir(data_dir).join(format!(".{}.installing", manifest.id));
    if temp.exists() { let _ = fs::remove_dir_all(&temp); }
    fs::create_dir_all(&temp).map_err(|e| e.to_string())?;

    let result = (|| -> Result<InstalledImage, String> {
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
            let Some(relative) = entry.enclosed_name() else { continue; };
            if relative == Path::new("manifest.json") { continue; }
            let output = temp.join(relative);
            if entry.is_dir() {
                fs::create_dir_all(&output).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = output.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
                let mut file = fs::File::create(&output).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut file).map_err(|e| e.to_string())?;
            }
        }

        let disk = temp.join(&manifest.disk);
        validate_manifest(&manifest, &disk)?;

        for optional in [
            manifest.boot_kernel.as_deref(),
            manifest.boot_initrd.as_deref(),
            manifest.vendor_disk.as_deref(),
        ].into_iter().flatten() {
            if !temp.join(optional).is_file() {
                return Err(format!("Android image package component is missing: {optional}"));
            }
        }

        manifest.source_url = Some(source_url.to_string());
        fs::write(temp.join("manifest.json"), serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

        if target.exists() { fs::remove_dir_all(&target).map_err(|e| e.to_string())?; }
        fs::rename(&temp, &target).map_err(|e| format!("Unable to install Android image package: {e}"))?;
        let installed_disk = target.join(&manifest.disk);

        Ok(InstalledImage {
            manifest,
            directory: target.to_string_lossy().to_string(),
            disk_path: installed_disk.to_string_lossy().to_string(),
            valid: true,
            validation_error: None,
        })
    })();

    if result.is_err() && temp.exists() { let _ = fs::remove_dir_all(&temp); }
    result
}

pub fn ensure_default_image_installed(data_dir: &Path) -> Result<InstalledImage, String> {
    if let Some(existing) = list_images(data_dir)?.into_iter()
        .filter(|image| image.valid)
        .find(|image| image.manifest.recommended)
    {
        return Ok(existing);
    }
    if let Some(existing) = list_images(data_dir)?.into_iter().find(|image| image.valid) {
        return Ok(existing);
    }

    let (url, sha) = resolve_default_image_url(data_dir, false)?;
    let package = download_to_cache(data_dir, &url, sha.as_deref())?;
    install_cached_image_package(data_dir, &package, &url)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct DefaultImageSettings {
    pub url:String,
    pub sha256:Option<String>,
    pub root_developer_url:Option<String>,
    pub root_developer_sha256:Option<String>,
}
fn default_image_settings_path(data_dir:&Path)->PathBuf{data_dir.join("default-image-settings.json")}
pub fn load_default_image_settings(data_dir:&Path)->Result<DefaultImageSettings,String>{
    let p=default_image_settings_path(data_dir);
    if !p.exists(){return Ok(DefaultImageSettings{url:String::new(),sha256:None,root_developer_url:None,root_developer_sha256:None});}
    serde_json::from_slice(&fs::read(p).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
}
pub fn save_default_image_settings(data_dir:&Path,s:DefaultImageSettings)->Result<DefaultImageSettings,String>{
    for url in [s.url.as_str(),s.root_developer_url.as_deref().unwrap_or("")] {
        if !url.is_empty() && !(url.starts_with("https://")||url.starts_with("http://127.0.0.1")||url.starts_with("http://localhost")){return Err("Default image URLs must use HTTPS (localhost HTTP allowed)".into());}
    }
    fs::write(default_image_settings_path(data_dir),serde_json::to_vec_pretty(&s).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;Ok(s)
}
fn default_manifest(root:bool,sha256:Option<String>)->AndroidImageManifest{
    AndroidImageManifest{
        id:if root{"android-16-x86_64-developer".into()}else{"android-16-x86_64-default".into()},
        name:if root{"Android 16 x86_64 Developer".into()}else{"Android 16 x86_64 Default".into()},
        android_version:"16".into(),api:36,architecture:"x86_64".into(),
        image_type:if root{"developer".into()}else{"aosp".into()},
        disk:if root{"android16-developer.qcow2".into()}else{"android16.qcow2".into()},
        disk_format:"qcow2".into(),recommended:!root,
        notes:Some(if root{"Root-capable developer image supplied by the owner/configured source.".into()}else{"Default Android image supplied by the owner/configured source.".into()}),
        sha256,source_url:None,gms_provider:"none".into(),certification_status:"unknown".into(),play_store_package:None,
        secure_image:false,verified_boot_state:"unknown".into(),security_state:if root{"developer/root-capable".into()}else{"virtualized".into()},
        missing_hardware_features:vec!["hardware-backed attestation".into()],
        boot_kernel:None,boot_initrd:None,vendor_disk:None,root_capable:root,
    }
}
pub fn download_default(data_dir:&Path,root:bool)->Result<InstalledImage,String>{
    let settings=load_default_image_settings(data_dir)?;
    let (url,sha)=if root{(settings.root_developer_url.ok_or("Root developer image URL is not configured")?,settings.root_developer_sha256)}else{
        if settings.url.trim().is_empty(){return Err("Default Android image URL is not configured".into());}
        (settings.url,settings.sha256)
    };
    download_image(data_dir,default_manifest(root,sha),url)
}

pub fn register_gsi_bundle(data_dir:&Path,android_version:String,system:String,kernel:String,initrd:String,vendor:Option<String>,root_capable:bool)->Result<InstalledImage,String>{
    ensure_layout(data_dir)?;
    for required in [&system,&kernel,&initrd]{if !Path::new(required).is_file(){return Err(format!("GSI bundle component missing: {required}"));}}
    if let Some(v)=vendor.as_deref(){if !Path::new(v).is_file(){return Err(format!("Vendor disk missing: {v}"));}}
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_secs();
    let id=format!("gsi-bundle-{}-{stamp}",android_version.replace('.',"-"));
    let target=images_dir(data_dir).join(&id);fs::create_dir_all(&target).map_err(|e|e.to_string())?;
    let system_name="system.img";let kernel_name="kernel";let initrd_name="ramdisk.img";
    fs::copy(&system,target.join(system_name)).map_err(|e|e.to_string())?;
    fs::copy(&kernel,target.join(kernel_name)).map_err(|e|e.to_string())?;
    fs::copy(&initrd,target.join(initrd_name)).map_err(|e|e.to_string())?;
    let vendor_name=if let Some(v)=vendor{fs::copy(v,target.join("vendor.img")).map_err(|e|e.to_string())?;Some("vendor.img".to_string())}else{None};
    let manifest=AndroidImageManifest{
        id:id.clone(),name:format!("Android {android_version} Custom GSI Boot Bundle"),android_version,api:0,architecture:"x86_64".into(),
        image_type:"gsi-bundle".into(),disk:system_name.into(),disk_format:"raw".into(),recommended:false,
        notes:Some("GSI boot bundle with explicit kernel, ramdisk and optional vendor disk.".into()),sha256:None,source_url:None,
        gms_provider:"none".into(),certification_status:"unknown".into(),play_store_package:None,secure_image:false,
        verified_boot_state:"unknown".into(),security_state:"custom GSI".into(),missing_hardware_features:Vec::new(),
        boot_kernel:Some(kernel_name.into()),boot_initrd:Some(initrd_name.into()),vendor_disk:vendor_name,root_capable,
    };
    fs::write(target.join("manifest.json"),serde_json::to_vec_pretty(&manifest).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(InstalledImage{manifest,directory:target.to_string_lossy().to_string(),disk_path:target.join(system_name).to_string_lossy().to_string(),valid:true,validation_error:None})
}
pub fn boot_component_args(data_dir:&Path,image_path:&str)->Result<Vec<String>,String>{
    let Some(image)=list_images(data_dir)?.into_iter().find(|i|i.disk_path==image_path) else{return Ok(Vec::new());};
    if image.manifest.image_type!="gsi-bundle"{return Ok(Vec::new());}
    let base=PathBuf::from(&image.directory);
    let kernel=image.manifest.boot_kernel.as_ref().ok_or("GSI bundle kernel missing from manifest")?;
    let initrd=image.manifest.boot_initrd.as_ref().ok_or("GSI bundle initrd missing from manifest")?;
    let mut args=vec!["-kernel".into(),base.join(kernel).to_string_lossy().to_string(),"-initrd".into(),base.join(initrd).to_string_lossy().to_string(),
        "-append".into(),"console=ttyS0 androidboot.hardware=goldfish androidboot.selinux=permissive".into()];
    if let Some(vendor)=image.manifest.vendor_disk.as_ref(){args.extend(["-drive".into(),format!("file={},if=virtio,format=raw,readonly=on",base.join(vendor).to_string_lossy())]);}
    Ok(args)
}
pub fn update_image(data_dir: &Path, id: &str) -> Result<InstalledImage, String> {
    // Image updates use the saved source URL and current manifest/checksum.
    // The replacement happens through the same temporary-file + verification
    // path as repair, so a failed download never replaces the installed disk.
    repair_image(data_dir, id)
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
