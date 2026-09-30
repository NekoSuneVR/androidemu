use crate::{adb,models::AdbResult};
use serde::{Deserialize,Serialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SensorState{
    pub battery_level:u8,
    pub charging:bool,
    pub latitude:f64,
    pub longitude:f64,
    pub altitude:f64,
}
impl Default for SensorState{fn default()->Self{Self{battery_level:100,charging:true,latitude:0.0,longitude:0.0,altitude:0.0}}}

pub fn battery(port:u16,level:u8,charging:bool)->Result<AdbResult,String>{
    if level>100{return Err("Battery level must be 0..100".into());}
    let status=if charging{2}else{3};
    adb::shell(port,format!("dumpsys battery set level {level}; dumpsys battery set status {status}; dumpsys battery set ac {}",if charging{1}else{0}))
}
pub fn reset_battery(port:u16)->Result<AdbResult,String>{adb::shell(port,"dumpsys battery reset".into())}
pub fn gps(port:u16,latitude:f64,longitude:f64,altitude:f64)->Result<AdbResult,String>{
    if !(-90.0..=90.0).contains(&latitude)||!(-180.0..=180.0).contains(&longitude){return Err("Invalid latitude/longitude".into());}
    let cmd=format!(
        "cmd location providers add-test-provider nekodroid --requiresNetwork --supportsAltitude --supportsSpeed 2>/dev/null || true; cmd location providers set-test-provider-enabled nekodroid true; cmd location providers set-test-provider-location nekodroid --location {latitude},{longitude} --accuracy 3 --time $(date +%s000) --altitude {altitude}"
    );
    adb::shell(port,cmd)
}
pub fn clear_gps(port:u16)->Result<AdbResult,String>{
    adb::shell(port,"cmd location providers set-test-provider-enabled nekodroid false 2>/dev/null; cmd location providers remove-test-provider nekodroid 2>/dev/null || true".into())
}
pub fn report(port:u16)->Result<AdbResult,String>{
    adb::shell(port,"echo '--- battery ---'; dumpsys battery; echo '--- location ---'; dumpsys location | head -n 160; echo '--- sensors ---'; dumpsys sensorservice | head -n 240".into())
}

fn bridge_broadcast(port:u16,action:&str,extras:&str)->Result<AdbResult,String>{
    adb::shell(port,format!("am broadcast -a uk.co.nekosunevr.nekodroid.bridge.{action} {extras}"))
}
pub fn accelerometer(port:u16,x:f64,y:f64,z:f64)->Result<AdbResult,String>{bridge_broadcast(port,"ACCELEROMETER",&format!("--ef x {x} --ef y {y} --ef z {z}"))}
pub fn gyroscope(port:u16,x:f64,y:f64,z:f64)->Result<AdbResult,String>{bridge_broadcast(port,"GYROSCOPE",&format!("--ef x {x} --ef y {y} --ef z {z}"))}
pub fn compass(port:u16,heading:f64)->Result<AdbResult,String>{bridge_broadcast(port,"COMPASS",&format!("--ef heading {heading}"))}
pub fn light(port:u16,lux:f64)->Result<AdbResult,String>{bridge_broadcast(port,"LIGHT",&format!("--ef lux {lux}"))}
pub fn proximity(port:u16,cm:f64)->Result<AdbResult,String>{bridge_broadcast(port,"PROXIMITY",&format!("--ef cm {cm}"))}
