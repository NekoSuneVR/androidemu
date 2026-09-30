mod models;

use models::{AndroidInstance, HostCapabilities};

#[tauri::command]
fn list_instances() -> Vec<AndroidInstance> {
    // Persistent instance storage lands next. Returning an empty list is intentional:
    // the UI can already distinguish "backend online" from "instance exists".
    Vec::new()
}

#[tauri::command]
fn get_host_capabilities() -> HostCapabilities {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    let (accelerator, virtualization_note) = match std::env::consts::OS {
        "windows" => (
            "WHPX / Hyper-V".to_string(),
            "Runtime probe not implemented yet".to_string(),
        ),
        "linux" => (
            "KVM".to_string(),
            "Runtime probe not implemented yet".to_string(),
        ),
        other => (
            "Software fallback".to_string(),
            format!("No native accelerator selected for {other}"),
        ),
    };

    HostCapabilities {
        os,
        arch,
        accelerator,
        virtualization_note,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_instances,
            get_host_capabilities
        ])
        .run(tauri::generate_context!())
        .expect("error while running NekoDroid");
}
