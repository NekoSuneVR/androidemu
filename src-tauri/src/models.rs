use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidInstance {
    pub id: String,
    pub name: String,
    pub android_version: String,
    pub profile: String,
    pub status: String,
    pub cpu_cores: u16,
    pub ram_mb: u32,
    pub adb_port: u16,
    #[serde(default)]
    pub adb_enabled: bool,
    #[serde(default)]
    pub headless: bool,
    pub root_mode: String,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub process_id: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceRequest {
    pub name: String,
    pub android_version: String,
    pub profile: String,
    pub cpu_cores: u16,
    pub ram_mb: u32,
    pub adb_port: u16,
    #[serde(default)]
    pub adb_enabled: bool,
    #[serde(default)]
    pub headless: bool,
    pub root_mode: String,
    pub image_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceRequest {
    pub name: String,
    pub adb_enabled: bool,
    pub headless: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostCapabilities {
    pub os: String,
    pub arch: String,
    pub accelerator: String,
    pub accelerator_available: bool,
    pub virtualization_note: String,
    pub qemu: QemuInfo,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QemuInfo {
    pub found: bool,
    pub executable: Option<String>,
    pub version: Option<String>,
    pub accelerators: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeActionResult {
    pub instance_id: String,
    pub status: String,
    pub message: String,
    pub process_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbInfo {
    pub found: bool,
    pub executable: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLogs {
    pub stdout: String,
    pub stderr: String,
    pub crash_report: String,
}
