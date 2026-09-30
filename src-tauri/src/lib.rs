mod adb;
mod images;
mod models;
mod profiles;
mod runtime;
mod storage;
mod settings;
mod snapshots;
mod media;
mod first_run;
mod automation_api;
mod ai;
mod skills;
mod keymaps;
mod game_settings;
mod performance;
mod transfer;
mod apk_bundle;
mod display;
mod graphics;
mod plugins;
mod sensors;
mod ai_capture;
mod media_jobs;
mod platform_tools;
mod android_validation;
mod remote_transfer;
mod host_media_io;
mod passthrough;

use images::{AndroidImageManifest, InstalledImage};
use models::{AdbInfo, AdbResult, AndroidInstance, CreateInstanceRequest, HostCapabilities, RuntimeActionResult, RuntimeLogs, UpdateInstanceRequest};
use adb::{AndroidFileEntry, ApkCompatibility};
use profiles::DeviceProfile;
use runtime::RuntimeState;
use settings::{AiSettings, AppSettings};
use snapshots::SnapshotInfo;
use media::{FfmpegInfo, MediaJobRequest, MediaResult, MediaCodecCapabilityReport};
use first_run::SystemReadiness;
use ai::{AiAction, AiChatResult};
use skills::{AiGameState, SkillManifest};
use keymaps::{KeyBinding, KeymapProfile};
use game_settings::GameSettings;
use performance::{PerformanceSettings,PerformanceTelemetry};
use transfer::TransferJob;
use apk_bundle::ApkPackageInfo;
use display::DisplayState;
use graphics::{GraphicsSettings,GraphicsCapabilities};
use plugins::PluginManifest;
use ai_capture::AiCaptureResult;
use media_jobs::{FfmpegSettings,MediaJobStatus};
use platform_tools::PlatformToolsSettings;
use passthrough::PassthroughSettings;
use android_validation::AndroidBootValidation;
use tauri::{Manager, State};
use std::{env, path::PathBuf};
use serde::Serialize;


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCheck {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_url: String,
}

#[tauri::command]
fn check_for_updates() -> Result<UpdateCheck, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let client = reqwest::blocking::Client::builder()
        .user_agent("NekoDroid update checker")
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get("https://api.github.com/repos/NekoSuneVR/androidemu/releases/latest")
        .send()
        .map_err(|e| format!("Unable to check GitHub Releases: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("GitHub Releases returned HTTP {}", response.status()));
    }

    let value: serde_json::Value = response.json().map_err(|e| e.to_string())?;
    let latest = value.get("tag_name").and_then(|v| v.as_str()).unwrap_or("").trim_start_matches('v').to_string();
    let release_url = value.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if latest.is_empty() {
        return Err("Latest release did not contain a version tag".into());
    }

    Ok(UpdateCheck {
        update_available: latest != current,
        current_version: current,
        latest_version: latest,
        release_url,
    })
}

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
fn update_instance(
    state: State<'_, RuntimeState>,
    id: String,
    request: UpdateInstanceRequest,
) -> Result<AndroidInstance, String> {
    storage::update_instance(&state.data_dir, &id, request)
}

#[tauri::command]
fn clone_instance(
    state: State<'_, RuntimeState>,
    id: String,
    name: String,
) -> Result<AndroidInstance, String> {
    if state.processes.lock().map_err(|_| "Runtime process lock poisoned")?.contains_key(&id) {
        return Err("Stop the instance before cloning it".into());
    }
    storage::clone_instance(&state.data_dir, &id, name)
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
fn validate_android_boot(port:u16,expected_version:Option<String>)->Result<AndroidBootValidation,String>{android_validation::validate(port,expected_version)}
#[tauri::command]
fn wait_android_boot(port:u16,expected_version:Option<String>,timeout_seconds:u32)->Result<AndroidBootValidation,String>{android_validation::wait_for_boot(port,expected_version,timeout_seconds)}
#[tauri::command]
fn test_google_services(port:u16)->Result<AdbResult,String>{android_validation::google_services_test(port)}
#[tauri::command]
fn test_google_login(port:u16)->Result<AdbResult,String>{android_validation::google_login_test(port)}

#[tauri::command]
fn get_platform_tools_settings(state:State<'_,RuntimeState>)->Result<PlatformToolsSettings,String>{platform_tools::load(&state.data_dir)}
#[tauri::command]
fn save_platform_tools_settings(state:State<'_,RuntimeState>,settings:PlatformToolsSettings)->Result<PlatformToolsSettings,String>{platform_tools::save(&state.data_dir,settings)}

#[tauri::command]
fn get_adb_info() -> AdbInfo {
    adb::detect_adb()
}

#[tauri::command]
fn adb_connect(port: u16) -> Result<AdbResult, String> {
    adb::connect(port)
}

#[tauri::command]
fn adb_disconnect(port: u16) -> Result<AdbResult, String> {
    adb::disconnect(port)
}

#[tauri::command]
fn adb_root(state: State<'_, RuntimeState>, port: u16) -> Result<AdbResult, String> {
    if let Ok(instances)=storage::load_instances(&state.data_dir) {
        if let Some(instance)=instances.into_iter().find(|item| item.adb_port==port && item.status!="running") {
            let _=snapshots::auto_snapshot(&state.data_dir,&instance.id,"pre-root");
        }
    }
    adb::root(port)
}

#[tauri::command]
fn adb_unroot(port: u16) -> Result<AdbResult, String> {
    adb::unroot(port)
}

#[tauri::command]
fn adb_root_shell(port: u16, command: String) -> Result<AdbResult, String> {
    adb::root_shell(port, command)
}

#[tauri::command]
fn adb_get_state(port: u16) -> Result<AdbResult, String> {
    adb::get_state(port)
}

#[tauri::command]
fn adb_shell(port: u16, command: String) -> Result<AdbResult, String> {
    adb::shell(port, command)
}


#[tauri::command]
fn inspect_apk_package(apk_path:String)->Result<ApkPackageInfo,String>{apk_bundle::inspect(apk_path)}
#[tauri::command]
fn install_apk_bundle(port:u16,bundle_path:String)->Result<Vec<AdbResult>,String>{apk_bundle::install_bundle(port,bundle_path)}

#[tauri::command]
fn inspect_apk(apk_path: String) -> Result<ApkCompatibility, String> {
    adb::inspect_apk(apk_path)
}

#[tauri::command]
fn adb_install(port: u16, apk_path: String) -> Result<AdbResult, String> {
    adb::install(port, apk_path)
}

#[tauri::command]
fn adb_install_batch(port: u16, apk_paths: Vec<String>) -> Result<AdbResult, String> {
    adb::install_batch(port, apk_paths)
}

#[tauri::command]
fn adb_install_multiple(port: u16, apk_paths: Vec<String>) -> Result<AdbResult, String> {
    adb::install_multiple(port, apk_paths)
}

#[tauri::command]
fn adb_uninstall(port: u16, package_name: String) -> Result<AdbResult, String> {
    adb::uninstall(port, package_name)
}

#[tauri::command]
fn adb_device_state(port:u16,state:Option<u32>)->Result<AdbResult,String>{adb::device_state(port,state)}

#[tauri::command]
fn adb_desktop_mode(port:u16,enabled:bool)->Result<AdbResult,String>{adb::desktop_mode(port,enabled)}
#[tauri::command]
fn adb_overlay_display(port:u16,spec:Option<String>)->Result<AdbResult,String>{adb::overlay_display(port,spec)}

#[tauri::command]
fn adb_timezone(port:u16,timezone:Option<String>)->Result<AdbResult,String>{adb::timezone(port,timezone)}

#[tauri::command]
fn adb_shutdown(port: u16) -> Result<AdbResult, String> { adb::shutdown(port) }

#[tauri::command]
fn adb_boot_status(port: u16) -> Result<AdbResult, String> { adb::boot_status(port) }

#[tauri::command]
fn adb_crash_diagnostics(port: u16) -> Result<AdbResult, String> { adb::crash_diagnostics(port) }

#[tauri::command]
fn adb_reboot(port: u16, mode: Option<String>) -> Result<AdbResult, String> {
    adb::reboot(port, mode)
}

#[tauri::command]
fn adb_device_info(port: u16) -> Result<AdbResult, String> {
    adb::device_info(port)
}

#[tauri::command]
fn receive_remote_file(port:u16,name:String,data_base64:String,destination:Option<String>)->Result<String,String>{
    remote_transfer::receive_and_push(port,name,data_base64,destination)
}

#[tauri::command]
fn adb_set_lan(port:u16,enabled:bool,lan_port:u16)->Result<AdbResult,String>{adb::set_lan_adb(port,enabled,lan_port)}

#[tauri::command]
fn adb_clipboard_set(port: u16, text: String) -> Result<AdbResult, String> { adb::clipboard_set(port, text) }
#[tauri::command]
fn adb_clipboard_get(port: u16) -> Result<AdbResult, String> { adb::clipboard_get(port) }

#[tauri::command]
fn adb_media_codec_requests(port: u16) -> Result<AdbResult, String> { adb::media_codec_requests(port) }

#[tauri::command]
fn adb_security_state(port: u16) -> Result<AdbResult, String> {
    adb::security_state(port)
}

#[tauri::command]
fn adb_screenshot(port: u16, destination: String) -> Result<AdbResult, String> {
    adb::screenshot(port, destination)
}

#[tauri::command]
fn adb_screen_record(
    port: u16,
    destination: String,
    seconds: u32,
) -> Result<AdbResult, String> {
    adb::screen_record(port, destination, seconds)
}

#[tauri::command]
fn adb_screen_record_advanced(port:u16,destination:String,seconds:u32,codec:String,bitrate_mbps:u32,fps:Option<u32>,audio:bool)->Result<AdbResult,String>{
    adb::screen_record_advanced(port,destination,seconds,codec,bitrate_mbps,fps,audio)
}


#[tauri::command]
fn start_transfer(port:u16,direction:String,source:String,destination:String)->Result<TransferJob,String>{transfer::start(port,direction,source,destination)}
#[tauri::command]
fn start_apk_install(port:u16,apk_path:String)->Result<TransferJob,String>{transfer::install(port,apk_path)}
#[tauri::command]
fn list_transfers()->Result<Vec<TransferJob>,String>{transfer::list()}
#[tauri::command]
fn get_transfer(id:String)->Result<TransferJob,String>{transfer::get(&id)}
#[tauri::command]
fn cancel_transfer(id:String)->Result<TransferJob,String>{transfer::cancel(&id)}
#[tauri::command]
fn retry_transfer(id:String)->Result<TransferJob,String>{transfer::retry(&id)}
#[tauri::command]
fn sync_shared_folder(port:u16,host_path:String)->Result<TransferJob,String>{transfer::sync_shared(port,host_path)}

#[tauri::command]
fn adb_push(port: u16, source: String, destination: String) -> Result<AdbResult, String> {
    adb::push(port, source, destination)
}

#[tauri::command]
fn adb_push_multiple(port: u16, sources: Vec<String>, destination: String) -> Result<AdbResult, String> {
    adb::push_multiple(port, sources, destination)
}

#[tauri::command]
fn adb_pull(port: u16, source: String, destination: String) -> Result<AdbResult, String> {
    adb::pull(port, source, destination)
}

#[tauri::command]
fn adb_forward(port: u16, local: String, remote: String) -> Result<AdbResult, String> {
    adb::forward(port, local, remote)
}

#[tauri::command]
fn adb_reverse(port: u16, remote: String, local: String) -> Result<AdbResult, String> {
    adb::reverse(port, remote, local)
}

#[tauri::command]
fn adb_logcat(port: u16, lines: u32) -> Result<AdbResult, String> {
    adb::logcat(port, lines)
}

#[tauri::command]
fn adb_packages(port: u16) -> Result<AdbResult, String> {
    adb::packages(port)
}

#[tauri::command]
fn adb_user_packages(port: u16) -> Result<Vec<String>, String> {
    adb::user_packages(port)
}

#[tauri::command]
fn adb_launch_package(port: u16, package_name: String) -> Result<AdbResult, String> {
    adb::launch_package(port, package_name)
}

#[tauri::command]
fn adb_processes(port: u16) -> Result<AdbResult, String> {
    adb::processes(port)
}

#[tauri::command]
fn adb_properties(port: u16) -> Result<AdbResult, String> {
    adb::properties(port)
}

#[tauri::command]
fn adb_build_properties(port: u16) -> Result<AdbResult, String> {
    adb::build_properties(port)
}

#[tauri::command]
fn adb_storage_info(port: u16) -> Result<AdbResult, String> {
    adb::storage_info(port)
}

#[tauri::command]
fn adb_activities(port: u16) -> Result<AdbResult, String> {
    adb::activities(port)
}

#[tauri::command]
fn adb_services(port: u16) -> Result<AdbResult, String> {
    adb::services(port)
}

#[tauri::command]
fn adb_network_connections(port: u16) -> Result<AdbResult, String> {
    adb::network_connections(port)
}

#[tauri::command]
fn adb_surfaceflinger_info(port: u16) -> Result<AdbResult, String> {
    adb::surfaceflinger_info(port)
}

#[tauri::command]
fn adb_kernel_log(port: u16, lines: u32) -> Result<AdbResult, String> {
    adb::kernel_log(port, lines)
}

#[tauri::command]
fn adb_input_tap(port: u16, x: i32, y: i32) -> Result<AdbResult, String> {
    adb::input_tap(port, x, y)
}

#[tauri::command]
fn adb_input_double_tap(port: u16, x: i32, y: i32) -> Result<AdbResult, String> {
    adb::input_double_tap(port, x, y)
}

#[tauri::command]
fn adb_input_hold(port: u16, x: i32, y: i32, duration_ms: u32) -> Result<AdbResult, String> {
    adb::input_hold(port, x, y, duration_ms)
}

#[tauri::command]
fn adb_input_swipe(
    port: u16,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    duration_ms: u32,
) -> Result<AdbResult, String> {
    adb::input_swipe(port, x1, y1, x2, y2, duration_ms)
}

#[tauri::command]
fn adb_input_keyevent(port: u16, keycode: String) -> Result<AdbResult, String> {
    adb::input_keyevent(port, keycode)
}

#[tauri::command]
fn adb_input_text(port: u16, text: String) -> Result<AdbResult, String> {
    adb::input_text(port, text)
}

#[tauri::command]
fn adb_list_files(port: u16, path: String) -> Result<Vec<AndroidFileEntry>, String> {
    adb::list_files(port, path)
}

#[tauri::command]
fn adb_make_directory(port: u16, path: String) -> Result<AdbResult, String> {
    adb::make_directory(port, path)
}

#[tauri::command]
fn adb_remove_path(port: u16, path: String) -> Result<AdbResult, String> {
    adb::remove_path(port, path)
}

#[tauri::command]
fn adb_move_path(port: u16, source: String, destination: String) -> Result<AdbResult, String> {
    adb::move_path(port, source, destination)
}

#[tauri::command]
fn adb_copy_path(port: u16, source: String, destination: String) -> Result<AdbResult, String> {
    adb::copy_path(port, source, destination)
}

#[tauri::command]
fn adb_file_properties(port: u16, path: String) -> Result<AdbResult, String> {
    adb::file_properties(port, path)
}

#[tauri::command]
fn adb_search_files(port: u16, path: String, query: String) -> Result<AdbResult, String> {
    adb::search_files(port, path, query)
}




#[tauri::command]
fn set_accelerometer(port:u16,x:f64,y:f64,z:f64)->Result<AdbResult,String>{sensors::accelerometer(port,x,y,z)}
#[tauri::command]
fn set_gyroscope(port:u16,x:f64,y:f64,z:f64)->Result<AdbResult,String>{sensors::gyroscope(port,x,y,z)}
#[tauri::command]
fn set_compass(port:u16,heading:f64)->Result<AdbResult,String>{sensors::compass(port,heading)}
#[tauri::command]
fn set_light_sensor(port:u16,lux:f64)->Result<AdbResult,String>{sensors::light(port,lux)}
#[tauri::command]
fn set_proximity_sensor(port:u16,cm:f64)->Result<AdbResult,String>{sensors::proximity(port,cm)}
#[tauri::command]
fn adb_input_multitouch(port:u16,points:Vec<(i32,i32)>,duration_ms:u32)->Result<AdbResult,String>{adb::input_multitouch(port,points,duration_ms)}
#[tauri::command]
fn adb_input_pinch(port:u16,cx:i32,cy:i32,from_radius:i32,to_radius:i32,duration_ms:u32)->Result<AdbResult,String>{adb::input_pinch(port,cx,cy,from_radius,to_radius,duration_ms)}

#[tauri::command]
fn set_battery_simulation(port:u16,level:u8,charging:bool)->Result<AdbResult,String>{sensors::battery(port,level,charging)}
#[tauri::command]
fn reset_battery_simulation(port:u16)->Result<AdbResult,String>{sensors::reset_battery(port)}
#[tauri::command]
fn set_gps_location(port:u16,latitude:f64,longitude:f64,altitude:f64)->Result<AdbResult,String>{sensors::gps(port,latitude,longitude,altitude)}
#[tauri::command]
fn clear_gps_location(port:u16)->Result<AdbResult,String>{sensors::clear_gps(port)}
#[tauri::command]
fn sensor_report(port:u16)->Result<AdbResult,String>{sensors::report(port)}

#[tauri::command]
fn get_display_state(port:u16)->Result<DisplayState,String>{display::state(port)}
#[tauri::command]
fn get_preferred_orientation(port:u16,package_name:String)->Result<AdbResult,String>{display::preferred_orientation(port,package_name)}

#[tauri::command]
fn adb_dynamic_resolution(port:u16,scale:f64)->Result<AdbResult,String>{adb::dynamic_resolution(port,scale)}

#[tauri::command]
fn adb_set_refresh_rate(port: u16, fps: Option<u32>) -> Result<AdbResult, String> {
    adb::set_refresh_rate(port, fps)
}

#[tauri::command]
fn adb_set_orientation(port: u16, orientation: String) -> Result<AdbResult, String> {
    adb::set_orientation(port, orientation)
}

#[tauri::command]
fn adb_rotate_orientation(port: u16, direction: String) -> Result<AdbResult, String> {
    adb::rotate_orientation(port, direction)
}

#[tauri::command]
fn list_device_profiles(state: State<'_, RuntimeState>) -> Result<Vec<DeviceProfile>, String> {
    profiles::list_profiles(&state.data_dir)
}

#[tauri::command]
fn save_device_profile(
    state: State<'_, RuntimeState>,
    profile: DeviceProfile,
) -> Result<DeviceProfile, String> {
    profiles::save_custom_profile(&state.data_dir, profile)
}

#[tauri::command]
fn remove_device_profile(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<(), String> {
    profiles::remove_custom_profile(&state.data_dir, &id)
}

#[tauri::command]
fn list_android_images(state: State<'_, RuntimeState>) -> Result<Vec<InstalledImage>, String> {
    images::list_images(&state.data_dir)
}

#[tauri::command]
fn supported_android_versions()->Vec<String>{images::supported_android_versions()}

#[tauri::command]
fn import_custom_gsi(state:State<'_,RuntimeState>,source:String,android_version:String,architecture:String)->Result<InstalledImage,String>{
    images::import_gsi(&state.data_dir,source,android_version,architecture)
}

#[tauri::command]
fn update_android_image(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<InstalledImage, String> {
    if let Ok(images)=images::list_images(&state.data_dir) {
        if let Some(image)=images.into_iter().find(|image| image.manifest.id==id) {
            if let Ok(instances)=storage::load_instances(&state.data_dir) {
                for instance in instances.into_iter().filter(|i| i.image_path.as_deref()==Some(image.disk_path.as_str()) && i.status!="running") {
                    let _=snapshots::auto_snapshot(&state.data_dir,&instance.id,"pre-update");
                }
            }
        }
    }
    images::update_image(&state.data_dir, &id)
}

#[tauri::command]
fn repair_android_image(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<InstalledImage, String> {
    images::repair_image(&state.data_dir, &id)
}

#[tauri::command]
fn remove_android_image(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<(), String> {
    images::remove_image(&state.data_dir, &id)
}

#[tauri::command]
fn download_android_image(
    state: State<'_, RuntimeState>,
    manifest: AndroidImageManifest,
    url: String,
) -> Result<InstalledImage, String> {
    images::download_image(&state.data_dir, manifest, url)
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
fn list_snapshots(
    state: State<'_, RuntimeState>,
    instance_id: String,
) -> Result<Vec<SnapshotInfo>, String> {
    snapshots::list(&state.data_dir, &instance_id)
}

#[tauri::command]
fn create_snapshot(
    state: State<'_, RuntimeState>,
    instance_id: String,
    name: String,
    description: String,
) -> Result<SnapshotInfo, String> {
    if state.processes.lock().map_err(|_| "Runtime process lock poisoned")?.contains_key(&instance_id) {
        return Err("Stop the instance before creating a snapshot".into());
    }
    snapshots::create(&state.data_dir, &instance_id, name, description)
}


#[tauri::command]
fn create_clean_snapshot(state: State<'_, RuntimeState>, instance_id:String)->Result<SnapshotInfo,String>{
    snapshots::create_clean(&state.data_dir,&instance_id)
}
#[tauri::command]
fn create_rooted_snapshot(state: State<'_, RuntimeState>, instance_id:String)->Result<SnapshotInfo,String>{
    snapshots::create_rooted(&state.data_dir,&instance_id)
}
#[tauri::command]
fn cleanup_snapshots(state: State<'_, RuntimeState>, instance_id:String, keep:usize)->Result<usize,String>{
    snapshots::cleanup(&state.data_dir,&instance_id,keep)
}

#[tauri::command]
fn rename_snapshot(
    state: State<'_, RuntimeState>,
    instance_id: String,
    snapshot_id: String,
    name: String,
    description: String,
) -> Result<SnapshotInfo, String> {
    if state.processes.lock().map_err(|_| "Runtime process lock poisoned")?.contains_key(&instance_id) {
        return Err("Stop the instance before renaming a snapshot".into());
    }
    snapshots::rename(&state.data_dir, &instance_id, &snapshot_id, name, description)
}

#[tauri::command]
fn restore_snapshot(
    state: State<'_, RuntimeState>,
    instance_id: String,
    snapshot_id: String,
) -> Result<(), String> {
    if state.processes.lock().map_err(|_| "Runtime process lock poisoned")?.contains_key(&instance_id) {
        return Err("Stop the instance before restoring a snapshot".into());
    }
    snapshots::restore(&state.data_dir, &instance_id, &snapshot_id)
}

#[tauri::command]
fn delete_snapshot(
    state: State<'_, RuntimeState>,
    instance_id: String,
    snapshot_id: String,
) -> Result<(), String> {
    if state.processes.lock().map_err(|_| "Runtime process lock poisoned")?.contains_key(&instance_id) {
        return Err("Stop the instance before deleting a snapshot".into());
    }
    snapshots::delete(&state.data_dir, &instance_id, &snapshot_id)
}


#[tauri::command]
fn repair_installation(state:State<'_,RuntimeState>)->Result<Vec<String>,String>{
    let mut actions=Vec::new();
    for name in ["images","instances","profiles","keymaps","game-settings","skills","adb"] {
        let path=state.data_dir.join(name);
        std::fs::create_dir_all(&path).map_err(|e|format!("Unable to repair {}: {e}",path.display()))?;
        actions.push(format!("Checked {}",path.display()));
    }
    images::ensure_layout(&state.data_dir)?;
    storage::ensure_layout(&state.data_dir).map_err(|e|e.to_string())?;
    if settings::load_app(&state.data_dir).is_err(){
        let path=state.data_dir.join("app-settings.json");
        if path.exists(){let backup=state.data_dir.join("app-settings.invalid.json");let _=std::fs::rename(&path,&backup);}
        settings::save_app(&state.data_dir,AppSettings::default())?;
        actions.push("Reset invalid app settings to safe defaults".into());
    }
    if settings::load(&state.data_dir).is_err(){
        let path=state.data_dir.join("ai-settings.json");
        if path.exists(){let backup=state.data_dir.join("ai-settings.invalid.json");let _=std::fs::rename(&path,&backup);}
        settings::save(&state.data_dir,AiSettings::default())?;
        actions.push("Reset invalid AI settings to safe defaults".into());
    }
    let readiness=first_run::detect();
    actions.push(format!("QEMU: {}",if readiness.qemu_found{"ready"}else{"missing"}));
    actions.push(format!("ADB: {}",if readiness.adb_found{"ready"}else{"missing"}));
    actions.push(format!("FFmpeg: {}",if readiness.ffmpeg_found{"ready"}else{"missing"}));
    Ok(actions)
}

#[tauri::command]
fn get_system_readiness() -> SystemReadiness {
    first_run::detect()
}



#[tauri::command]
fn capture_android_audio(port:u16,destination:String,seconds:u32)->Result<MediaResult,String>{host_media_io::android_audio_capture(port,destination,seconds)}
#[tauri::command]
fn mix_microphone(input:String,output:String,microphone:String)->Result<MediaResult,String>{host_media_io::mix_microphone(input,output,microphone)}
#[tauri::command]
fn output_virtual_camera(input:String,device:String)->Result<MediaResult,String>{host_media_io::virtual_camera(input,device)}

#[tauri::command]
fn get_ffmpeg_settings(state:State<'_,RuntimeState>)->Result<FfmpegSettings,String>{media_jobs::load_settings(&state.data_dir)}
#[tauri::command]
fn save_ffmpeg_settings(state:State<'_,RuntimeState>,settings:FfmpegSettings)->Result<FfmpegSettings,String>{media_jobs::save_settings(&state.data_dir,settings)}
#[tauri::command]
fn start_media_job(state:State<'_,RuntimeState>,request:MediaJobRequest)->Result<MediaJobStatus,String>{media_jobs::start(state.data_dir.clone(),request)}
#[tauri::command]
fn list_media_jobs()->Result<Vec<MediaJobStatus>,String>{media_jobs::list()}
#[tauri::command]
fn cancel_media_job(id:String)->Result<MediaJobStatus,String>{media_jobs::cancel(&id)}

#[tauri::command]
fn get_media_codec_report() -> MediaCodecCapabilityReport { media::media_codec_report() }

#[tauri::command]
fn get_ffmpeg_info() -> FfmpegInfo {
    media::detect_ffmpeg()
}

#[tauri::command]
fn run_media_job(
    input: String,
    output: String,
    operation: String,
    video_codec: String,
    audio_codec: String,
    width: Option<u32>,
    height: Option<u32>,
    fps: Option<u32>,
    hardware_decode: bool,
) -> Result<MediaResult, String> {
    media::run_media_job(
        input,
        output,
        operation,
        video_codec,
        audio_codec,
        width,
        height,
        fps,
        hardware_decode,
    )
}

#[tauri::command]
fn stream_media(input:String,url:String,video_codec:String,rotate:Option<String>)->Result<MediaResult,String>{
    media::stream_media(input,url,video_codec,rotate)
}




#[tauri::command]
fn execute_plugin(state:State<'_,RuntimeState>,id:String,input:serde_json::Value)->Result<serde_json::Value,String>{plugins::execute(&state.data_dir,id,input)}
#[tauri::command]
fn run_renderer_plugins(state:State<'_,RuntimeState>,input:serde_json::Value)->Result<Vec<serde_json::Value>,String>{plugins::renderer_hook(&state.data_dir,input)}
#[tauri::command]
fn run_ai_plugins(state:State<'_,RuntimeState>,input:serde_json::Value)->Result<Vec<serde_json::Value>,String>{plugins::ai_hook(&state.data_dir,input)}

#[tauri::command]
fn list_plugins(state:State<'_,RuntimeState>)->Result<Vec<PluginManifest>,String>{plugins::list(&state.data_dir)}
#[tauri::command]
fn save_plugin(state:State<'_,RuntimeState>,plugin:PluginManifest)->Result<PluginManifest,String>{plugins::save(&state.data_dir,plugin)}
#[tauri::command]
fn import_plugin(state:State<'_,RuntimeState>,path:String)->Result<PluginManifest,String>{plugins::import_file(&state.data_dir,path)}
#[tauri::command]
fn remove_plugin(state:State<'_,RuntimeState>,id:String)->Result<(),String>{plugins::remove(&state.data_dir,id)}


#[tauri::command]
fn get_passthrough_settings(state:State<'_,RuntimeState>)->Result<PassthroughSettings,String>{passthrough::load(&state.data_dir)}
#[tauri::command]
fn save_passthrough_settings(state:State<'_,RuntimeState>,settings:PassthroughSettings)->Result<PassthroughSettings,String>{passthrough::save(&state.data_dir,settings)}

#[tauri::command]
fn get_graphics_settings(state:State<'_,RuntimeState>)->Result<GraphicsSettings,String>{graphics::load(&state.data_dir)}
#[tauri::command]
fn save_graphics_settings(state:State<'_,RuntimeState>,settings:GraphicsSettings)->Result<GraphicsSettings,String>{graphics::save(&state.data_dir,settings)}
#[tauri::command]
fn get_graphics_capabilities()->GraphicsCapabilities{graphics::detect()}
#[tauri::command]
fn run_graphics_benchmark()->Result<Vec<String>,String>{graphics::benchmark()}
#[tauri::command]
fn astc_transcode(input:String,output:String,decode:bool)->Result<String,String>{graphics::astc_transcode(input,output,decode)}
#[tauri::command]
fn astc_cache_transcode(state:State<'_,RuntimeState>,input:String,decode:bool)->Result<String,String>{graphics::astc_cache_transcode(&state.data_dir,input,decode)}
#[tauri::command]
fn run_performance_benchmark(state:State<'_,RuntimeState>)->Result<Vec<String>,String>{performance::benchmark_builtin(&state.data_dir)}

#[tauri::command]
fn get_performance_telemetry()->PerformanceTelemetry{performance::telemetry()}

#[tauri::command]
fn get_performance_settings(state: State<'_, RuntimeState>) -> Result<PerformanceSettings, String> {
    performance::load(&state.data_dir)
}
#[tauri::command]
fn save_performance_settings(state: State<'_, RuntimeState>, settings: PerformanceSettings) -> Result<PerformanceSettings, String> {
    performance::save(&state.data_dir, settings)
}

#[tauri::command]
fn get_app_settings(state: State<'_, RuntimeState>) -> Result<AppSettings, String> {
    settings::load_app(&state.data_dir)
}

#[tauri::command]
fn save_app_settings(
    state: State<'_, RuntimeState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    settings::save_app(&state.data_dir, settings)
}

#[tauri::command]
fn run_media_batch(jobs: Vec<MediaJobRequest>) -> Result<Vec<MediaResult>, String> {
    media::run_media_batch(jobs)
}




#[tauri::command]
fn list_game_settings(state: State<'_, RuntimeState>) -> Result<Vec<GameSettings>, String> {
    game_settings::list(&state.data_dir)
}
#[tauri::command]
fn save_game_settings(state: State<'_, RuntimeState>, settings: GameSettings) -> Result<GameSettings, String> {
    game_settings::save(&state.data_dir, settings)
}
#[tauri::command]
fn remove_game_settings(state: State<'_, RuntimeState>, package_name: String) -> Result<(), String> {
    game_settings::remove(&state.data_dir, package_name)
}
#[tauri::command]
fn launch_game_with_settings(
    state: State<'_, RuntimeState>,
    port: u16,
    package_name: String,
) -> Result<Vec<AdbResult>, String> {
    let mut out=Vec::new();
    if let Some(settings)=game_settings::list(&state.data_dir)?.into_iter().find(|s| s.package_name==package_name) {
        out.extend(game_settings::apply(port,&settings)?);
    }
    out.push(adb::launch_package(port,package_name)?);
    Ok(out)
}

#[tauri::command]
fn list_keymaps(state: State<'_, RuntimeState>) -> Result<Vec<KeymapProfile>, String> {
    keymaps::list(&state.data_dir)
}
#[tauri::command]
fn save_keymap(state: State<'_, RuntimeState>, profile: KeymapProfile) -> Result<KeymapProfile, String> {
    keymaps::save(&state.data_dir, profile)
}
#[tauri::command]
fn remove_keymap(state: State<'_, RuntimeState>, id: String) -> Result<(), String> {
    keymaps::remove(&state.data_dir, &id)
}
#[tauri::command]
fn import_keymap(state: State<'_, RuntimeState>, source: String) -> Result<KeymapProfile, String> {
    keymaps::import_file(&state.data_dir, source)
}
#[tauri::command]
fn export_keymap(state: State<'_, RuntimeState>, id: String, destination: String) -> Result<String, String> {
    keymaps::export_file(&state.data_dir, id, destination)
}
#[tauri::command]
fn execute_key_binding(port: u16, binding: KeyBinding) -> Result<AdbResult, String> {
    keymaps::execute(port, binding)
}


#[tauri::command]
fn ai_capture_frame(port:u16,destination:String,max_fps:u32)->Result<AiCaptureResult,String>{ai_capture::capture(port,destination,max_fps)}
#[tauri::command]
fn ai_capture_roi(port:u16,destination:String,x:u32,y:u32,width:u32,height:u32,max_fps:u32)->Result<AiCaptureResult,String>{ai_capture::capture_roi(port,destination,x,y,width,height,max_fps)}
#[tauri::command]
fn ai_ocr(image_path:String)->Result<String,String>{ai_capture::ocr(image_path)}
#[tauri::command]
fn ai_speech_to_text(audio_path:String)->Result<String,String>{ai_capture::speech_to_text(audio_path)}
#[tauri::command]
fn ai_tts(text:String)->Result<String,String>{ai_capture::tts(text)}
#[tauri::command]
fn ai_input_visualizer(port:u16)->Result<String,String>{ai_capture::input_visualizer(port)}

#[tauri::command]
fn list_ai_skills(state: State<'_, RuntimeState>) -> Result<Vec<SkillManifest>, String> {
    skills::list(&state.data_dir)
}

#[tauri::command]
fn save_ai_skill(state: State<'_, RuntimeState>, skill: SkillManifest) -> Result<SkillManifest, String> {
    skills::save(&state.data_dir, skill)
}

#[tauri::command]
fn import_ai_skill(state: State<'_, RuntimeState>, source: String) -> Result<SkillManifest, String> {
    skills::import_file(&state.data_dir, source)
}

#[tauri::command]
fn export_ai_skill(state: State<'_, RuntimeState>, id: String, destination: String) -> Result<String, String> {
    skills::export_file(&state.data_dir, id, destination)
}

#[tauri::command]
fn get_ai_game_state(port: u16) -> Result<AiGameState, String> {
    skills::game_state(port)
}

#[tauri::command]
fn ai_execute_actions(
    state: State<'_, RuntimeState>,
    port: u16,
    actions: Vec<AiAction>,
) -> Result<Vec<AdbResult>, String> {
    ai::execute_actions(&state.data_dir, port, actions)
}

#[tauri::command]
fn ai_cancel_actions() {
    ai::cancel_actions();
}

#[tauri::command]
fn ai_execute_action(
    state: State<'_, RuntimeState>,
    port: u16,
    action: AiAction,
) -> Result<AdbResult, String> {
    ai::execute_action(&state.data_dir, port, action)
}

#[tauri::command]
fn ai_chat(
    state: State<'_, RuntimeState>,
    prompt: String,
) -> Result<AiChatResult, String> {
    ai::chat(&state.data_dir, prompt)
}

#[tauri::command]
fn get_ai_logs(state: State<'_, RuntimeState>) -> Result<String, String> {
    ai::read_logs(&state.data_dir)
}

#[tauri::command]
fn clear_ai_logs(state: State<'_, RuntimeState>) -> Result<(), String> {
    ai::clear_logs(&state.data_dir)
}

#[tauri::command]
fn get_instance_ai_settings(state:State<'_,RuntimeState>,instance_id:String)->Result<AiSettings,String>{settings::load_for_instance(&state.data_dir,&instance_id)}
#[tauri::command]
fn save_instance_ai_settings(state:State<'_,RuntimeState>,instance_id:String,settings:AiSettings)->Result<AiSettings,String>{settings::save_for_instance(&state.data_dir,&instance_id,settings)}

#[tauri::command]
fn get_ai_settings(state: State<'_, RuntimeState>) -> Result<AiSettings, String> {
    settings::load(&state.data_dir)
}

#[tauri::command]
fn save_ai_settings(
    state: State<'_, RuntimeState>,
    settings: AiSettings,
) -> Result<AiSettings, String> {
    settings::save(&state.data_dir, settings)
}

#[tauri::command]
fn ai_emergency_stop(state: State<'_, RuntimeState>) -> Result<AiSettings, String> {
    settings::emergency_stop(&state.data_dir)
}

#[tauri::command]
fn get_host_capabilities() -> HostCapabilities {
    runtime::detect_host()
}

#[tauri::command]
fn set_root_on_next_boot(state:State<'_,RuntimeState>,id:String,enabled:bool)->Result<(),String>{
    if state.processes.lock().map_err(|_|"Runtime process lock poisoned")?.contains_key(&id){return Err("Stop the instance before changing root-on-next-boot".into());}
    runtime::set_root_on_next_boot(&state,&id,enabled)
}

#[tauri::command]
fn start_instance(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<RuntimeActionResult, String> {
    runtime::start_instance(&state, &id)
}

#[tauri::command]
fn pause_instance(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<RuntimeActionResult, String> {
    runtime::pause_instance(&state, &id)
}

#[tauri::command]
fn resume_instance(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<RuntimeActionResult, String> {
    runtime::resume_instance(&state, &id)
}

#[tauri::command]
fn stop_instance(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<RuntimeActionResult, String> {
    runtime::stop_instance(&state, &id)
}

#[tauri::command]
fn factory_reset_instance(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<RuntimeActionResult, String> {
    runtime::factory_reset(&state, &id)
}

#[tauri::command]
fn capture_instance_framebuffer_base64(state:State<'_,RuntimeState>,id:String)->Result<String,String>{runtime::capture_framebuffer_base64(&state,&id)}

#[tauri::command]
fn capture_instance_framebuffer(state:State<'_,RuntimeState>,id:String,destination:String)->Result<String,String>{
    runtime::capture_framebuffer(&state,&id,destination)
}
#[tauri::command]
fn record_instance_framebuffer(state:State<'_,RuntimeState>,id:String,destination:String,seconds:u32,fps:u32)->Result<String,String>{
    runtime::record_framebuffer(&state,&id,destination,seconds,fps)
}

#[tauri::command]
fn get_instance_logs(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<RuntimeLogs, String> {
    runtime::read_logs(&state, &id)
}

#[tauri::command]
fn get_instance_status(
    state: State<'_, RuntimeState>,
    id: String,
) -> Result<AndroidInstance, String> {
    runtime::runtime_status(&state, &id)
}

fn resolve_data_dir(app: &tauri::App) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let portable = env::var("NEKODROID_PORTABLE")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);

    if portable {
        let exe = env::current_exe()?;
        let base = exe.parent()
            .ok_or_else(|| std::io::Error::other("Unable to determine executable directory"))?;
        return Ok(base.join("NekoDroidData"));
    }

    Ok(app.path().app_data_dir()?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = resolve_data_dir(app)?;
            if let Ok(tool_settings)=platform_tools::load(&data_dir) {
                if let Err(error)=platform_tools::apply(&tool_settings){eprintln!("Unable to apply platform-tools settings: {error}");}
            }
            if let Err(error) = adb::configure_adb_keys(&data_dir) {
                eprintln!("Unable to configure dedicated ADB keys: {error}");
            }
            let runtime = RuntimeState::new(data_dir).map_err(std::io::Error::other)?;

            match settings::load_app(&runtime.data_dir) {
                Ok(app_settings) if app_settings.api_enabled => {
                    if let Err(error) = automation_api::start(runtime.clone(), app_settings) {
                        eprintln!("Automation API did not start: {error}");
                    }
                }
                Ok(_) => {}
                Err(error) => eprintln!("Unable to load automation API settings: {error}"),
            }

            app.manage(runtime);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_for_updates,
            list_instances,
            validate_android_boot,
            wait_android_boot,
            test_google_services,
            test_google_login,
            get_platform_tools_settings,
            save_platform_tools_settings,
            get_adb_info,
            adb_connect,
            adb_disconnect,
            adb_root,
            adb_unroot,
            adb_root_shell,
            adb_get_state,
            adb_shell,
            inspect_apk_package,
            install_apk_bundle,
            inspect_apk,
            adb_install,
            adb_install_batch,
            adb_install_multiple,
            adb_uninstall,
            adb_device_state,
            adb_desktop_mode,
            adb_overlay_display,
            adb_timezone,
            adb_shutdown,
            adb_boot_status,
            adb_crash_diagnostics,
            adb_reboot,
            adb_device_info,
            receive_remote_file,
            adb_set_lan,
            adb_clipboard_set,
            adb_clipboard_get,
            adb_media_codec_requests,
            adb_security_state,
            adb_screenshot,
            adb_screen_record,
            start_transfer,
            start_apk_install,
            list_transfers,
            get_transfer,
            cancel_transfer,
            retry_transfer,
            sync_shared_folder,
            adb_screen_record_advanced,
            adb_push,
            adb_push_multiple,
            adb_pull,
            adb_forward,
            adb_reverse,
            adb_logcat,
            adb_packages,
            adb_user_packages,
            adb_launch_package,
            adb_processes,
            adb_properties,
            adb_build_properties,
            adb_storage_info,
            adb_activities,
            adb_services,
            adb_network_connections,
            adb_surfaceflinger_info,
            adb_kernel_log,
            adb_input_tap,
            adb_input_double_tap,
            adb_input_hold,
            adb_input_swipe,
            adb_input_keyevent,
            adb_input_text,
            adb_list_files,
            adb_make_directory,
            adb_remove_path,
            adb_move_path,
            adb_copy_path,
            adb_file_properties,
            adb_search_files,
            set_accelerometer,
            set_gyroscope,
            set_compass,
            set_light_sensor,
            set_proximity_sensor,
            adb_input_multitouch,
            adb_input_pinch,
            set_battery_simulation,
            reset_battery_simulation,
            set_gps_location,
            clear_gps_location,
            sensor_report,
            get_display_state,
            get_preferred_orientation,
            adb_dynamic_resolution,
            adb_set_refresh_rate,
            adb_set_orientation,
            adb_rotate_orientation,
            create_instance,
            update_instance,
            clone_instance,
            delete_instance,
            list_device_profiles,
            save_device_profile,
            remove_device_profile,
            list_android_images,
            supported_android_versions,
            import_custom_gsi,
            update_android_image,
            repair_android_image,
            remove_android_image,
            download_android_image,
            register_android_image,
            list_snapshots,
            create_snapshot,
            create_clean_snapshot,
            create_rooted_snapshot,
            cleanup_snapshots,
            rename_snapshot,
            restore_snapshot,
            delete_snapshot,
            repair_installation,
            get_system_readiness,
            capture_android_audio,
            mix_microphone,
            output_virtual_camera,
            get_ffmpeg_settings,
            save_ffmpeg_settings,
            start_media_job,
            list_media_jobs,
            cancel_media_job,
            get_media_codec_report,
            get_ffmpeg_info,
            run_media_job,
            run_media_batch,
            stream_media,
            execute_plugin,
            run_renderer_plugins,
            run_ai_plugins,
            list_plugins,
            save_plugin,
            import_plugin,
            remove_plugin,
            get_passthrough_settings,
            save_passthrough_settings,
            get_graphics_settings,
            save_graphics_settings,
            get_graphics_capabilities,
            run_graphics_benchmark,
            astc_transcode,
            astc_cache_transcode,
            run_performance_benchmark,
            get_performance_telemetry,
            get_performance_settings,
            save_performance_settings,
            get_app_settings,
            save_app_settings,
            list_game_settings,
            save_game_settings,
            remove_game_settings,
            launch_game_with_settings,
            list_keymaps,
            save_keymap,
            remove_keymap,
            import_keymap,
            export_keymap,
            execute_key_binding,
            ai_capture_frame,
            ai_capture_roi,
            ai_ocr,
            ai_speech_to_text,
            ai_tts,
            ai_input_visualizer,
            list_ai_skills,
            save_ai_skill,
            import_ai_skill,
            export_ai_skill,
            get_ai_game_state,
            ai_execute_actions,
            ai_cancel_actions,
            ai_execute_action,
            ai_chat,
            get_ai_logs,
            clear_ai_logs,
            get_instance_ai_settings,
            save_instance_ai_settings,
            get_ai_settings,
            save_ai_settings,
            ai_emergency_stop,
            get_host_capabilities,
            set_root_on_next_boot,
            start_instance,
            pause_instance,
            resume_instance,
            stop_instance,
            factory_reset_instance,
            capture_instance_framebuffer_base64,
            capture_instance_framebuffer,
            record_instance_framebuffer,
            get_instance_logs,
            get_instance_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running NekoDroid");
}
