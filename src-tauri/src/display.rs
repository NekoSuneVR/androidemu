use crate::{adb,models::AdbResult};
use serde::Serialize;

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct DisplayState {
    pub width:i32,
    pub height:i32,
    pub orientation:String,
    pub rotation:u8,
}
pub fn state(port:u16)->Result<DisplayState,String>{
    let size=adb::shell(port,"wm size".into())?;
    let dims=size.stdout.lines().find_map(|l|l.split_once(':').map(|(_,v)|v.trim())).unwrap_or("1080x2400");
    let (w,h)=dims.split_once('x').and_then(|(a,b)|Some((a.trim().parse().ok()?,b.trim().parse().ok()?))).unwrap_or((1080,2400));
    let orient=adb::shell(port,"dumpsys input | grep -m1 'SurfaceOrientation'".into())?;
    let rotation=orient.stdout.split(':').last().and_then(|v|v.trim().parse::<u8>().ok()).unwrap_or(0)%4;
    let orientation=if rotation%2==1{"landscape"}else{"portrait"}.to_string();
    Ok(DisplayState{width:w,height:h,orientation,rotation})
}
pub fn transform(port:u16,x:i32,y:i32)->Result<(i32,i32),String>{
    let s=state(port)?;
    let (tx,ty)=match s.rotation{
        1=>(s.height-1-y,x),
        2=>(s.width-1-x,s.height-1-y),
        3=>(y,s.width-1-x),
        _=>(x,y),
    };
    Ok((tx.max(0),ty.max(0)))
}
pub fn transform_swipe(port:u16,x1:i32,y1:i32,x2:i32,y2:i32)->Result<(i32,i32,i32,i32),String>{
    let a=transform(port,x1,y1)?;let b=transform(port,x2,y2)?;Ok((a.0,a.1,b.0,b.1))
}
pub fn preferred_orientation(port:u16,package:String)->Result<AdbResult,String>{
    adb::shell(port,format!("dumpsys package {package} | grep -Ei 'screenOrientation|resizeMode|supports.*screen' | head -n 40"))
}
