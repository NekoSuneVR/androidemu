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
    pub root_mode: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostCapabilities {
    pub os: String,
    pub arch: String,
    pub accelerator: String,
    pub virtualization_note: String,
}
