use libloading::{Library,Symbol};
use serde::Serialize;
use std::ffi::CStr;

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct SdlControllerInfo{pub index:i32,pub name:String,pub is_game_controller:bool}

pub fn list()->Result<Vec<SdlControllerInfo>,String>{
    let names:&[&str]=if cfg!(windows){&["SDL2.dll"]}else if cfg!(target_os="macos"){&["libSDL2.dylib"]}else{&["libSDL2-2.0.so.0","libSDL2.so"]};
    let lib=names.iter().find_map(|name|unsafe{Library::new(name).ok()}).ok_or("SDL2 shared library not found on host")?;
    unsafe{
        type Init=unsafe extern "C" fn(u32)->i32;
        type Quit=unsafe extern "C" fn();
        type Num=unsafe extern "C" fn()->i32;
        type Is=unsafe extern "C" fn(i32)->i32;
        type Name=unsafe extern "C" fn(i32)->*const i8;
        let init:Symbol<Init>=lib.get(b"SDL_Init\0").map_err(|e|e.to_string())?;
        let quit:Symbol<Quit>=lib.get(b"SDL_Quit\0").map_err(|e|e.to_string())?;
        let num:Symbol<Num>=lib.get(b"SDL_NumJoysticks\0").map_err(|e|e.to_string())?;
        let isgc:Symbol<Is>=lib.get(b"SDL_IsGameController\0").map_err(|e|e.to_string())?;
        let name:Symbol<Name>=lib.get(b"SDL_JoystickNameForIndex\0").map_err(|e|e.to_string())?;
        if init(0x00002000)!=0{return Err("SDL2 game-controller subsystem failed to initialize".into());}
        let mut out=Vec::new();
        for index in 0..num(){
            let ptr=name(index);
            let label=if ptr.is_null(){format!("Controller {index}")}else{CStr::from_ptr(ptr).to_string_lossy().to_string()};
            out.push(SdlControllerInfo{index,name:label,is_game_controller:isgc(index)!=0});
        }
        quit();Ok(out)
    }
}
