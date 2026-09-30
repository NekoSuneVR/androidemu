use serde::{Deserialize,Serialize};
use std::{env,fs,path::{Path,PathBuf},process::Command};

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",default)]
pub struct GraphicsSettings{
    pub renderer:String,
    pub vsync:bool,
    pub triple_buffer:bool,
    pub dynamic_resolution:bool,
    pub shader_cache:bool,
    pub astc_cache:bool,
    pub fps_overlay:bool,
    pub frame_time_graph:bool,
    pub usage_overlay:bool,
}
impl Default for GraphicsSettings{
    fn default()->Self{Self{renderer:"auto".into(),vsync:true,triple_buffer:true,dynamic_resolution:false,shader_cache:true,astc_cache:true,fps_overlay:false,frame_time_graph:false,usage_overlay:false}}
}
#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct GraphicsCapabilities{
    pub virgl:bool,
    pub angle:bool,
    pub vulkan:bool,
    pub opengl:bool,
    pub directx:bool,
    pub software:bool,
    pub astc_tools:Vec<String>,
}
fn path(data_dir:&Path)->PathBuf{data_dir.join("graphics-settings.json")}
pub fn load(data_dir:&Path)->Result<GraphicsSettings,String>{
    let p=path(data_dir);if !p.exists(){return Ok(GraphicsSettings::default());}
    serde_json::from_slice(&fs::read(p).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
}
pub fn save(data_dir:&Path,s:GraphicsSettings)->Result<GraphicsSettings,String>{
    if !matches!(s.renderer.as_str(),"auto"|"vulkan"|"opengl"|"directx"|"virgl"|"software"){return Err("Invalid renderer mode".into());}
    fs::write(path(data_dir),serde_json::to_vec_pretty(&s).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;Ok(s)
}
fn has(name:&str)->bool{env::var_os("PATH").and_then(|paths|env::split_paths(&paths).map(|d|d.join(name)).find(|p|p.is_file())).is_some()}
pub fn detect()->GraphicsCapabilities{
    let vulkan=has(if cfg!(windows){"vulkaninfo.exe"}else{"vulkaninfo"});
    let opengl=if cfg!(windows){true}else{has("glxinfo")||Path::new("/usr/lib").exists()};
    let directx=cfg!(windows);
    let angle=env::var_os("ANGLE_DEFAULT_PLATFORM").is_some()||has(if cfg!(windows){"libEGL.dll"}else{"libEGL.so"});
    let virgl=has(if cfg!(windows){"virgl_test_server.exe"}else{"virgl_test_server"})||has("virglrenderer");
    let astc_tools=["astcenc","astcenc-avx2","astcenc-sse2"].into_iter().filter(|n|has(n)).map(str::to_string).collect();
    GraphicsCapabilities{virgl,angle,vulkan,opengl,directx,software:true,astc_tools}
}
pub fn qemu_video_device(s:&GraphicsSettings)->(&'static str,bool){
    match s.renderer.as_str(){
        "virgl"|"vulkan"|"opengl"|"directx"=>("virtio-gpu-gl",true),
        "software"=>("virtio-vga",false),
        _=>("virtio-gpu-gl",true),
    }
}
pub fn launch_env(s:&GraphicsSettings)->Vec<(String,String)>{
    let mut out=Vec::new();
    if s.shader_cache{out.push(("MESA_SHADER_CACHE_DISABLE".into(),"false".into()));}
    if s.astc_cache{out.push(("NEKODROID_ASTC_CACHE".into(),"1".into()));}
    out.push(("NEKODROID_VSYNC".into(),if s.vsync{"1".into()}else{"0".into()}));
    out.push(("vblank_mode".into(),if s.vsync{"1".into()}else{"0".into()}));
    out.push(("__GL_SYNC_TO_VBLANK".into(),if s.vsync{"1".into()}else{"0".into()}));
    out.push(("NEKODROID_TRIPLE_BUFFER".into(),if s.triple_buffer{"1".into()}else{"0".into()}));
    out.push(("__GL_TRIPLE_BUFFER".into(),if s.triple_buffer{"1".into()}else{"0".into()}));
    out.push(("NEKODROID_DYNAMIC_RESOLUTION".into(),if s.dynamic_resolution{"1".into()}else{"0".into()}));
    match s.renderer.as_str(){
        "vulkan"=>{out.push(("ANGLE_DEFAULT_PLATFORM".into(),"vulkan".into()));},
        "directx"=>{out.push(("ANGLE_DEFAULT_PLATFORM".into(),"d3d11".into()));},
        "opengl"=>{out.push(("ANGLE_DEFAULT_PLATFORM".into(),"gl".into()));},
        _=>{}
    }
    out
}
pub fn benchmark()->Result<Vec<String>,String>{
    let mut out=Vec::new();
    if let Ok(result)=Command::new(if cfg!(windows){"vulkaninfo.exe"}else{"vulkaninfo"}).arg("--summary").output(){
        out.push(String::from_utf8_lossy(&result.stdout).lines().take(40).collect::<Vec<_>>().join("\n"));
    }
    Ok(out)
}

pub fn astc_transcode(input:String,output:String,decode:bool)->Result<String,String>{
    let exe=["astcenc","astcenc-avx2","astcenc-sse2"].into_iter()
        .find_map(|name|env::var_os("PATH").and_then(|paths|env::split_paths(&paths).map(|d|d.join(name)).find(|p|p.is_file())))
        .ok_or("astcenc was not found in PATH")?;
    if !Path::new(&input).is_file(){return Err("ASTC input file does not exist".into());}
    if let Some(parent)=Path::new(&output).parent(){if !parent.as_os_str().is_empty(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}}
    let args=if decode{vec!["-dl".to_string(),input,output.clone()]}else{vec!["-cl".to_string(),input,output.clone(),"6x6".into(),"-medium".into()]};
    let result=Command::new(exe).args(args).output().map_err(|e|e.to_string())?;
    if !result.status.success(){return Err(String::from_utf8_lossy(&result.stderr).to_string());}
    Ok(output)
}
