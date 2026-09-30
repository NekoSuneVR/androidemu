use crate::{adb, models::AdbResult};
use serde::Serialize;
use std::{env,fs,path::{Path,PathBuf},process::Command,time::{SystemTime,UNIX_EPOCH}};

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct ApkPackageInfo {
    pub path:String,
    pub package_name:Option<String>,
    pub version_name:Option<String>,
    pub version_code:Option<String>,
    pub abis:Vec<String>,
    pub size_bytes:u64,
}

fn find_tool(names:&[&str])->Option<PathBuf>{
    env::var_os("PATH").and_then(|paths|names.iter().find_map(|name|env::split_paths(&paths).map(|d|d.join(name)).find(|p|p.is_file())))
}

pub fn inspect(path:String)->Result<ApkPackageInfo,String>{
    let p=Path::new(&path); if !p.is_file(){return Err("APK does not exist".into());}
    let abi=adb::inspect_apk(path.clone())?;
    let mut package_name=None;let mut version_name=None;let mut version_code=None;
    if let Some(aapt)=find_tool(if cfg!(windows){&["aapt2.exe","aapt.exe","aapt2","aapt"]}else{&["aapt2","aapt"]}){
        if let Ok(out)=Command::new(aapt).args(["dump","badging",&path]).output(){
            let text=String::from_utf8_lossy(&out.stdout);
            if let Some(line)=text.lines().find(|l|l.starts_with("package:")){
                for part in line.split_whitespace(){
                    if let Some(v)=part.strip_prefix("name='").and_then(|v|v.strip_suffix('\'')){package_name=Some(v.into());}
                    if let Some(v)=part.strip_prefix("versionName='").and_then(|v|v.strip_suffix('\'')){version_name=Some(v.into());}
                    if let Some(v)=part.strip_prefix("versionCode='").and_then(|v|v.strip_suffix('\'')){version_code=Some(v.into());}
                }
            }
        }
    }
    Ok(ApkPackageInfo{path,package_name,version_name,version_code,abis:abi.abis,size_bytes:fs::metadata(p).map_err(|e|e.to_string())?.len()})
}

pub fn install_bundle(port:u16,bundle_path:String)->Result<Vec<AdbResult>,String>{
    let p=Path::new(&bundle_path);if !p.is_file(){return Err("Bundle does not exist".into());}
    let ext=p.extension().and_then(|v|v.to_str()).unwrap_or("").to_ascii_lowercase();
    if ext=="apk"{return Ok(vec![adb::install(port,bundle_path)?]);}
    if !matches!(ext.as_str(),"apks"|"xapk"|"zip"){return Err("Bundle must be APK, APKS, XAPK, or ZIP".into());}
    let file=fs::File::open(p).map_err(|e|e.to_string())?;
    let mut zip=zip::ZipArchive::new(file).map_err(|e|format!("Invalid APK bundle: {e}"))?;
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    let tmp=env::temp_dir().join(format!("nekodroid-apk-{stamp}"));fs::create_dir_all(&tmp).map_err(|e|e.to_string())?;
    let mut apks=Vec::new();let mut obbs=Vec::new();
    for i in 0..zip.len(){
        let mut entry=zip.by_index(i).map_err(|e|e.to_string())?;
        if entry.is_dir(){continue;}
        let name=entry.name().replace('\\',"/");
        if name.ends_with(".apk"){
            let out=tmp.join(Path::new(&name).file_name().ok_or("Invalid APK filename")?);
            let mut f=fs::File::create(&out).map_err(|e|e.to_string())?;std::io::copy(&mut entry,&mut f).map_err(|e|e.to_string())?;apks.push(out);
        }else if ext=="xapk" && name.contains("/obb/") && name.ends_with(".obb"){
            let rel=name.split_once("Android/obb/").map(|(_,r)|r.to_string()).or_else(||name.split_once("obb/").map(|(_,r)|r.to_string()));
            if let Some(rel)=rel{
                let out=tmp.join(Path::new(&rel).file_name().ok_or("Invalid OBB filename")?);
                let mut f=fs::File::create(&out).map_err(|e|e.to_string())?;std::io::copy(&mut entry,&mut f).map_err(|e|e.to_string())?;obbs.push((out,rel));
            }
        }
    }
    if apks.is_empty(){return Err("Bundle contains no APK files".into());}
    let apk_strings=apks.iter().map(|p|p.to_string_lossy().to_string()).collect::<Vec<_>>();
    let mut results=vec![if apk_strings.len()==1{adb::install(port,apk_strings[0].clone())?}else{adb::install_multiple(port,apk_strings)?}];
    if results[0].success{
        for (obb,rel) in obbs{
            let package=rel.split('/').next().unwrap_or("");
            if package.is_empty(){continue;}
            let dir=format!("/sdcard/Android/obb/{package}");
            results.push(adb::shell(port,format!("mkdir -p '{}'",dir.replace('\'',"")))?);
            results.push(adb::push(port,obb.to_string_lossy().to_string(),format!("{dir}/"))?);
        }
    }
    let _=fs::remove_dir_all(tmp);
    Ok(results)
}
