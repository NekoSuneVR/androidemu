mod images;
mod models;
mod profiles;
mod runtime;
mod storage;

use images::{AndroidImageManifest, InstalledImage};
use models::{AndroidInstance, CreateInstanceRequest, HostCapabilities, RuntimeActionResult};
use profiles::DeviceProfile;
use runtime::RuntimeState;
use tauri::{Manager, State};

#[tauri::command]
fn list_instances(state: State<'_, RuntimeState>) -> Result<Vec<AndroidInstance>, String> {
    runtime::refresh_processes(&state)?;
    storage::load_instances(&state.data_dir)
}

#[tauri::command]
fn create_instance(
    state: State<'_, RuntimeState>,
    request: CreateInstanceRequest,
) -> Result<AndroidInstance, String> {
    storage::create_instance(&state.data_dir, request)
}

#[tauri::command]
fn delete_instance(state: State<'_, RuntimeState>, id: String) -> Result<(), String> {
    let running = state
        .processes
        .lock()
        .map_err(|_| "Runtime process lock poisoned")?
        .contains_key(&id);

    if running {
        return Err("Stop the instance before deleting it".into());
    }

    storage::delete_instance(&state.data_dir, &id)
}


#[tauri::command]
fn list_device_profiles() -> Vec<DeviceProfile> {
    profiles::builtin_profiles()
}

#[tauri::command]
fn list_android_images(state: State<'_, RuntimeState>) -> Result<Vec<InstalledImage>, String> {
    images::list_images(&state.data_dir)
}

#[tauri::command]
fn register_android_image(
    state: State<'_, RuntimeState>,
    manifest: AndroidImageManifest,
    source_disk: String,
) -> Result<InstalledImage, String> {
    images::register_image(&state.data_dir, manifest, &source_disk)
}

#[tauri::command]
fn get_host_capabilities() -> HostCapabilities {
    runtime::detect_host()
}

#[tauri::command]
fn start_instance(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<RuntimeActionResult, String> {
    runtime::start_instance(&state, &id)
}

#[tauri::command]
fn stop_instance(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<RuntimeActionResult, String> {
    runtime::stop_instance(&state, &id)
}

#[tauri::command]
fn get_instance_status(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<AndroidInstance, String> {
    runtime::runtime_status(&state, &id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(RuntimeState::new(data_dir).map_err(std::io::Error::other)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_instances,
            create_instance,
            delete_instance,
            list_device_profiles,
            list_android_images,
            register_android_image,
            get_host_capabilities,
            start_instance,
            stop_instance,
            get_instance_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running NekoDroid");
}
