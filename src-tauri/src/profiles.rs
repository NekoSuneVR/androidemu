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
        },
    ]
}


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
