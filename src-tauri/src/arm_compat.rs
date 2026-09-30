use crate::{adb,models::AdbResult};
use std::{env,fs,path::PathBuf,time::{SystemTime,UNIX_EPOCH}};

pub fn install_native_bridge(port:u16,archive:String)->Result<Vec<AdbResult>,String>{
    let path=PathBuf::from(&archive);
    if !path.is_file(){return Err("Native-bridge package does not exist".into());}
    let ext=path.extension().and_then(|v|v.to_str()).unwrap_or("").to_ascii_lowercase();
    if !matches!(ext.as_str(),"zip"|"tgz"|"gz"){return Err("Native-bridge package must be zip/tgz/gz and contain install.sh".into());}
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    let remote=format!("/data/local/tmp/nekodroid-nativebridge-{stamp}.{ext}");
    let mut out=Vec::new();
    out.push(adb::push(port,archive,remote.clone())?);
    if !out.last().map(|r|r.success).unwrap_or(false){return Ok(out);}
    let dir=format!("/data/local/tmp/nekodroid-nativebridge-{stamp}");
    let extract=if ext=="zip"{format!("mkdir -p {dir}; unzip -o {remote} -d {dir}")}else{format!("mkdir -p {dir}; tar -xf {remote} -C {dir}")};
    out.push(adb::root_shell(port,extract)?);
    if !out.last().map(|r|r.success).unwrap_or(false){return Ok(out);}
    out.push(adb::root_shell(port,format!("test -f {dir}/install.sh && chmod 700 {dir}/install.sh && {dir}/install.sh --nekodroid || (echo 'Native bridge package must contain install.sh' >&2; exit 2)"))?);
    out.push(adb::root_shell(port,"getprop ro.dalvik.vm.native.bridge; getprop ro.enable.native.bridge.exec; getprop ro.product.cpu.abilist".into())?);
    let _=adb::root_shell(port,format!("rm -rf {dir} {remote}"));
    Ok(out)
}

pub fn compatibility_test(port:u16,abi:String)->Result<AdbResult,String>{
    if !matches!(abi.as_str(),"arm64-v8a"|"armeabi-v7a"){return Err("ABI must be arm64-v8a or armeabi-v7a".into());}
    adb::shell(port,format!("echo requested_abi={abi}; getprop ro.dalvik.vm.native.bridge; getprop ro.product.cpu.abilist; pm list packages -3 | head -n 100"))
}
