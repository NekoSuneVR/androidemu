use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PerformanceSettings {
    pub cpu_affinity: String,
    pub huge_pages: bool,
    pub io_mode: String,
    pub ram_compression: bool,
    pub disk_cache: String,
    pub shader_cache: bool,
    pub audio_latency_ms: u32,
    pub input_latency_mode: String,
    pub frame_pacing: String,
    pub startup_optimization: bool,
    pub minimize_background_services: bool,
    pub cpu_overlay: bool,
    pub gpu_overlay: bool,
    pub ram_overlay: bool,
}
impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            cpu_affinity: String::new(),
            huge_pages: false,
            io_mode: "native".into(),
            ram_compression: false,
            disk_cache: "writeback".into(),
            shader_cache: true,
            audio_latency_ms: 40,
            input_latency_mode: "balanced".into(),
            frame_pacing: "balanced".into(),
            startup_optimization: true,
            minimize_background_services: false,
            cpu_overlay: false,
            gpu_overlay: false,
            ram_overlay: false,
        }
    }
}
fn path(data_dir:&Path)->PathBuf{data_dir.join("performance-settings.json")}
pub fn load(data_dir:&Path)->Result<PerformanceSettings,String>{
    let p=path(data_dir); if !p.exists(){return Ok(PerformanceSettings::default());}
    let bytes=fs::read(&p).map_err(|e|e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e|format!("Invalid performance settings: {e}"))
}
pub fn save(data_dir:&Path,s:PerformanceSettings)->Result<PerformanceSettings,String>{
    if !matches!(s.io_mode.as_str(),"native"|"threads"){return Err("I/O mode must be native or threads".into());}
    if !matches!(s.disk_cache.as_str(),"none"|"writeback"|"writethrough"|"directsync"){return Err("Invalid disk cache mode".into());}
    if !matches!(s.input_latency_mode.as_str(),"balanced"|"low-latency"){return Err("Invalid input latency mode".into());}
    if !matches!(s.frame_pacing.as_str(),"balanced"|"smooth"|"low-latency"){return Err("Invalid frame pacing mode".into());}
    if !(10..=250).contains(&s.audio_latency_ms){return Err("Audio latency must be 10..250 ms".into());}
    fs::create_dir_all(data_dir).map_err(|e|e.to_string())?;
    fs::write(path(data_dir),serde_json::to_vec_pretty(&s).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(s)
}
pub fn qemu_drive_options(s:&PerformanceSettings)->String{
    let aio=if s.io_mode=="threads"{"threads"}else{"native"};
    format!("cache={},aio={aio}",s.disk_cache)
}
pub fn launch_env(s:&PerformanceSettings)->Vec<(String,String)>{
    let mut out=Vec::new();
    if s.shader_cache {
        out.push(("MESA_SHADER_CACHE_DISABLE".into(),"false".into()));
        out.push(("MESA_SHADER_CACHE_MAX_SIZE".into(),"1G".into()));
    }
    out.push(("NEKODROID_AUDIO_LATENCY_MS".into(),s.audio_latency_ms.to_string()));
    out.push(("NEKODROID_INPUT_LATENCY_MODE".into(),s.input_latency_mode.clone()));
    out.push(("NEKODROID_FRAME_PACING".into(),s.frame_pacing.clone()));
    out
}
