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

use images::{AndroidImageManifest, InstalledImage};
use models::{AdbInfo, AdbResult, AndroidInstance, CreateInstanceRequest, HostCapabilities, RuntimeActionResult, RuntimeLogs, UpdateInstanceRequest};
use adb::{AndroidFileEntry, ApkCompatibility};
use profiles::DeviceProfile;
use runtime::RuntimeState;
use settings::{AiSettings, AppSettings};
use snapshots::SnapshotInfo;
use media::{FfmpegInfo, MediaJobRequest, MediaResult};
use first_run::SystemReadiness;
use ai::{AiAction, AiChatResult};
use skills::{AiGameState, SkillManifest};
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
fn adb_root(port: u16) -> Result<AdbResult, String> {
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
fn adb_reboot(port: u16, mode: Option<String>) -> Result<AdbResult, String> {
    adb::reboot(port, mode)
}

#[tauri::command]
fn adb_device_info(port: u16) -> Result<AdbResult, String> {
    adb::device_info(port)
}

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
fn get_system_readiness() -> SystemReadiness {
    first_run::detect()
}

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
            get_adb_info,
            adb_connect,
            adb_disconnect,
            adb_root,
            adb_unroot,
            adb_root_shell,
            adb_get_state,
            adb_shell,
            inspect_apk,
            adb_install,
            adb_install_batch,
            adb_install_multiple,
            adb_uninstall,
            adb_reboot,
            adb_device_info,
            adb_security_state,
            adb_screenshot,
            adb_screen_record,
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
            repair_android_image,
            remove_android_image,
            download_android_image,
            register_android_image,
            list_snapshots,
            create_snapshot,
            rename_snapshot,
            restore_snapshot,
            delete_snapshot,
            get_system_readiness,
            get_ffmpeg_info,
            run_media_job,
            run_media_batch,
            get_app_settings,
            save_app_settings,
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
            get_ai_settings,
            save_ai_settings,
            ai_emergency_stop,
            get_host_capabilities,
            start_instance,
            pause_instance,
            resume_instance,
            stop_instance,
            factory_reset_instance,
            get_instance_logs,
            get_instance_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running NekoDroid");
}
