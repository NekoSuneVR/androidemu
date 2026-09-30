use crate::{adb,models::AdbResult};
use serde::Serialize;
use std::{thread,time::Duration};

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct AndroidBootValidation{
    pub boot_completed:bool,
    pub launcher_ready:bool,
    pub system_ui_ready:bool,
    pub boot_animation_stopped:bool,
    pub android_version:String,
    pub architecture:String,
    pub details:String,
}
pub fn validate(port:u16,expected_version:Option<String>)->Result<AndroidBootValidation,String>{
    let boot=adb::shell(port,"getprop sys.boot_completed".into())?;
    let version=adb::shell(port,"getprop ro.build.version.release".into())?;
    let arch=adb::shell(port,"getprop ro.product.cpu.abi".into())?;
    let anim=adb::shell(port,"getprop init.svc.bootanim".into())?;
    let launcher=adb::shell(port,"cmd package resolve-activity --brief -a android.intent.action.MAIN -c android.intent.category.HOME 2>/dev/null".into())?;
    let systemui=adb::shell(port,"pidof com.android.systemui 2>/dev/null || true".into())?;
    let current=version.stdout.trim().to_string();
    let expected_ok=expected_version.as_deref().map(|v|current.starts_with(v)).unwrap_or(true);
    Ok(AndroidBootValidation{
        boot_completed:boot.stdout.trim()=="1"&&expected_ok,
        launcher_ready:launcher.success&&!launcher.stdout.trim().is_empty()&&!launcher.stdout.contains("No activity"),
        system_ui_ready:!systemui.stdout.trim().is_empty(),
        boot_animation_stopped:anim.stdout.trim()=="stopped",
        android_version:current,
        architecture:arch.stdout.trim().to_string(),
        details:format!("boot={} launcher={} systemui={} bootanim={}",boot.stdout.trim(),launcher.stdout.trim(),systemui.stdout.trim(),anim.stdout.trim()),
    })
}
pub fn wait_for_boot(port:u16,expected_version:Option<String>,timeout_seconds:u32)->Result<AndroidBootValidation,String>{
    let timeout=timeout_seconds.clamp(10,600);
    for _ in 0..timeout/2 {
        let _=adb::connect(port);
        if let Ok(v)=validate(port,expected_version.clone()){
            if v.boot_completed&&v.launcher_ready&&v.system_ui_ready&&v.boot_animation_stopped{return Ok(v);}
        }
        thread::sleep(Duration::from_secs(2));
    }
    validate(port,expected_version)
}
pub fn google_services_test(port:u16)->Result<AdbResult,String>{
    adb::shell(port,"echo '--- packages ---'; pm path com.google.android.gms; pm path com.android.vending; echo '--- account types ---'; dumpsys account | grep -Ei 'com.google|Account {' | head -n 80; echo '--- GMS version ---'; dumpsys package com.google.android.gms | grep -m1 versionName; echo '--- Play Store version ---'; dumpsys package com.android.vending | grep -m1 versionName".into())
}
pub fn google_login_test(port:u16)->Result<AdbResult,String>{
    adb::shell(port,"echo '--- Google account presence ---'; dumpsys account | grep -Ei 'type=com.google|com.google' | head -n 80; echo '--- auth services ---'; dumpsys activity services com.google.android.gms | grep -Ei 'auth|signin' | head -n 80".into())
}
