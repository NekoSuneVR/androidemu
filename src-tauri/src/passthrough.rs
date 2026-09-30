use serde::{Deserialize,Serialize};
use std::{fs,path::{Path,PathBuf}};

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",default)]
pub struct PassthroughSettings{
    pub usb_devices:Vec<String>,
    pub bluetooth_usb_device:String,
    pub webcam_usb_device:String,
}
impl Default for PassthroughSettings{
    fn default()->Self{Self{usb_devices:Vec::new(),bluetooth_usb_device:String::new(),webcam_usb_device:String::new()}}
}
fn path(data_dir:&Path)->PathBuf{data_dir.join("passthrough-settings.json")}
pub fn load(data_dir:&Path)->Result<PassthroughSettings,String>{
    let p=path(data_dir);if !p.exists(){return Ok(PassthroughSettings::default());}
    serde_json::from_slice(&fs::read(p).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
}
pub fn save(data_dir:&Path,s:PassthroughSettings)->Result<PassthroughSettings,String>{
    for spec in s.usb_devices.iter().chain([&s.bluetooth_usb_device,&s.webcam_usb_device]){
        if spec.is_empty(){continue;}
        if !spec.split(':').all(|v|v.len()==4&&v.chars().all(|c|c.is_ascii_hexdigit())){return Err(format!("USB device must be VVVV:PPPP, got {spec}"));}
    }
    fs::write(path(data_dir),serde_json::to_vec_pretty(&s).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;Ok(s)
}
pub fn qemu_args(s:&PassthroughSettings)->Vec<String>{
    let mut out=Vec::new(); let mut all=s.usb_devices.clone();
    if !s.bluetooth_usb_device.is_empty(){all.push(s.bluetooth_usb_device.clone());}
    if !s.webcam_usb_device.is_empty(){all.push(s.webcam_usb_device.clone());}
    if all.is_empty(){return out;}
    out.extend(["-device".into(),"qemu-xhci,id=usb".into()]);
    for (index,spec) in all.into_iter().enumerate(){
        let Some((vendor,product))=spec.split_once(':') else{continue};
        out.extend(["-device".into(),format!("usb-host,bus=usb.0,port={},vendorid=0x{},productid=0x{}",index+1,vendor,product)]);
    }
    out
}
