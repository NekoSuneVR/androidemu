use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceProfile {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    pub refresh_rate: u32,
    pub default_cpu_cores: u16,
    pub default_ram_mb: u32,
    pub touch_points: u8,
    pub telephony: bool,
    pub form_factor: String,
    #[serde(default = "default_storage_gb")]
    pub storage_gb: u32,
    #[serde(default = "default_true")]
    pub wifi: bool,
    #[serde(default = "default_true")]
    pub bluetooth: bool,
    #[serde(default = "default_true")]
    pub gps: bool,
    #[serde(default = "default_camera_configuration")]
    pub camera_configuration: String,
    #[serde(default = "default_true")]
    pub microphone: bool,
    #[serde(default = "default_true")]
    pub accelerometer: bool,
    #[serde(default = "default_true")]
    pub gyroscope: bool,
    #[serde(default = "default_true")]
    pub compass: bool,
    #[serde(default = "default_true")]
    pub light_sensor: bool,
    #[serde(default = "default_true")]
    pub proximity_sensor: bool,
    #[serde(default = "default_battery_percent")]
    pub battery_percent: u8,
    #[serde(default)]
    pub charging: bool,
    #[serde(default)]
    pub tablet_resources: bool,
}

pub fn builtin_profiles() -> Vec<DeviceProfile> {
    vec![
        DeviceProfile {
            id: "phone".into(),
            name: "Phone".into(),
            width: 1080,
            height: 2400,
            dpi: 420,
            refresh_rate: 60,
            default_cpu_cores: 4,
            default_ram_mb: 4096,
            touch_points: 10,
            telephony: true,
            form_factor: "phone".into(),
            storage_gb: 64,
            wifi: true,
            bluetooth: true,
            gps: true,
            camera_configuration: "front+rear".into(),
            microphone: true,
            accelerometer: true,
            gyroscope: true,
            compass: true,
            light_sensor: true,
            proximity_sensor: true,
            battery_percent: 100,
            charging: false,
            tablet_resources: false,
        },
        DeviceProfile {
            id: "gaming-phone".into(),
            name: "Gaming Phone".into(),
            width: 1080,
            height: 2400,
            dpi: 420,
            refresh_rate: 120,
            default_cpu_cores: 8,
            default_ram_mb: 8192,
            touch_points: 10,
            telephony: true,
            form_factor: "phone".into(),
            storage_gb: 64,
            wifi: true,
            bluetooth: true,
            gps: true,
            camera_configuration: "front+rear".into(),
            microphone: true,
            accelerometer: true,
            gyroscope: true,
            compass: true,
            light_sensor: true,
            proximity_sensor: true,
            battery_percent: 100,
            charging: false,
            tablet_resources: false,
        },
        DeviceProfile {
            id: "tablet".into(),
            name: "Tablet".into(),
            width: 2560,
            height: 1600,
            dpi: 280,
            refresh_rate: 90,
            default_cpu_cores: 6,
            default_ram_mb: 6144,
            touch_points: 10,
            telephony: false,
            form_factor: "tablet".into(),
            storage_gb: 64,
            wifi: true,
            bluetooth: true,
            gps: true,
            camera_configuration: "front+rear".into(),
            microphone: true,
            accelerometer: true,
            gyroscope: true,
            compass: true,
            light_sensor: true,
            proximity_sensor: true,
            battery_percent: 100,
            charging: false,
            tablet_resources: true,
        },
        DeviceProfile {
            id: "large-tablet".into(),
            name: "Large Tablet".into(),
            width: 2560,
            height: 1600,
            dpi: 240,
            refresh_rate: 120,
            default_cpu_cores: 8,
            default_ram_mb: 8192,
            touch_points: 10,
            telephony: false,
            form_factor: "tablet".into(),
            storage_gb: 64,
            wifi: true,
            bluetooth: true,
            gps: true,
            camera_configuration: "front+rear".into(),
            microphone: true,
            accelerometer: true,
            gyroscope: true,
            compass: true,
            light_sensor: true,
            proximity_sensor: true,
            battery_percent: 100,
            charging: false,
            tablet_resources: true,
        },
        DeviceProfile {
            id: "foldable".into(),
            name: "Foldable".into(),
            width: 2208,
            height: 1840,
            dpi: 420,
            refresh_rate: 120,
            default_cpu_cores: 8,
            default_ram_mb: 8192,
            touch_points: 10,
            telephony: true,
            form_factor: "foldable".into(),
            storage_gb: 64,
            wifi: true,
            bluetooth: true,
            gps: true,
            camera_configuration: "front+rear".into(),
            microphone: true,
            accelerometer: true,
            gyroscope: true,
            compass: true,
            light_sensor: true,
            proximity_sensor: true,
            battery_percent: 100,
            charging: false,
            tablet_resources: false,
        },
    ]
}

fn default_true() -> bool { true }
fn default_storage_gb() -> u32 { 64 }
fn default_battery_percent() -> u8 { 100 }
fn default_camera_configuration() -> String { "front+rear".into() }

fn custom_profiles_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("profiles")
}

pub fn list_profiles(data_dir: &Path) -> Result<Vec<DeviceProfile>, String> {
    let mut profiles = builtin_profiles();
    let dir = custom_profiles_dir(data_dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(entry.path()).map_err(|e| e.to_string())?;
        match serde_json::from_slice::<DeviceProfile>(&bytes) {
            Ok(profile) => profiles.push(profile),
            Err(error) => eprintln!("Skipping invalid custom profile {}: {error}", entry.path().display()),
        }
    }

    profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(profiles)
}

pub fn save_custom_profile(data_dir: &Path, profile: DeviceProfile) -> Result<DeviceProfile, String> {
    validate_profile(&profile)?;
    if builtin_profiles().iter().any(|item| item.id == profile.id) {
        return Err("Custom profile ID conflicts with a built-in profile".into());
    }

    let dir = custom_profiles_dir(data_dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let encoded = serde_json::to_vec_pretty(&profile).map_err(|e| e.to_string())?;
    fs::write(dir.join(format!("{}.json", profile.id)), encoded).map_err(|e| e.to_string())?;
    Ok(profile)
}

pub fn remove_custom_profile(data_dir: &Path, id: &str) -> Result<(), String> {
    if builtin_profiles().iter().any(|item| item.id == id) {
        return Err("Built-in profiles cannot be removed".into());
    }
    validate_id(id)?;
    let path = custom_profiles_dir(data_dir).join(format!("{id}.json"));
    if !path.exists() {
        return Err("Custom profile does not exist".into());
    }
    fs::remove_file(path).map_err(|e| e.to_string())
}

fn validate_profile(profile: &DeviceProfile) -> Result<(), String> {
    validate_id(&profile.id)?;
    if profile.name.trim().is_empty() {
        return Err("Profile name cannot be empty".into());
    }
    if !(320..=8192).contains(&profile.width) || !(320..=8192).contains(&profile.height) {
        return Err("Resolution must be between 320 and 8192 pixels per side".into());
    }
    if !(72..=1000).contains(&profile.dpi) {
        return Err("DPI must be between 72 and 1000".into());
    }
    if !(30..=360).contains(&profile.refresh_rate) {
        return Err("Refresh rate must be between 30 and 360 Hz".into());
    }
    if !(1..=64).contains(&profile.default_cpu_cores) {
        return Err("CPU cores must be between 1 and 64".into());
    }
    if !(512..=131_072).contains(&profile.default_ram_mb) {
        return Err("RAM must be between 512 MB and 128 GB".into());
    }
    if !(1..=20).contains(&profile.touch_points) {
        return Err("Touch points must be between 1 and 20".into());
    }
    if !(4..=2048).contains(&profile.storage_gb) {
        return Err("Storage must be between 4 GB and 2048 GB".into());
    }
    if profile.battery_percent > 100 {
        return Err("Battery percentage must be between 0 and 100".into());
    }
    if !matches!(profile.camera_configuration.as_str(), "none" | "front" | "rear" | "front+rear") {
        return Err("Camera configuration must be none, front, rear, or front+rear".into());
    }
    if !matches!(profile.form_factor.as_str(), "phone" | "tablet" | "foldable") {
        return Err("Form factor must be phone, tablet, or foldable".into());
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("Profile ID may contain only letters, numbers, '-' and '_'".into());
    }
    Ok(())
}
