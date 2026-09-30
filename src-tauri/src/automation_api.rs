use crate::{adb, runtime, sensors, settings, storage};
use crate::runtime::RuntimeState;
use crate::settings::AppSettings;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};

pub fn start(state: RuntimeState, settings: AppSettings) -> Result<(), String> {
    if !settings.api_enabled {
        return Ok(());
    }

    if settings.api_token.len() < 16 {
        return Err("Automation API token must be at least 16 characters".into());
    }

    let listener = TcpListener::bind(("127.0.0.1", settings.api_port))
        .map_err(|e| format!("Unable to bind localhost automation API: {e}"))?;

    let token = settings.api_token;
    let ws_state = state.clone();
    let ws_token = token.clone();
    let ws_port = settings.api_port.saturating_add(1);
    thread::spawn(move || {
        if let Ok(listener)=TcpListener::bind(("127.0.0.1",ws_port)) {
            for stream in listener.incoming().flatten() {
                let state=ws_state.clone();
                let token=ws_token.clone();
                thread::spawn(move||{let _=handle_websocket(stream,&state,&token);});
            }
        }
    });

    thread::spawn(move || {
        for incoming in listener.incoming() {
            match incoming {
                Ok(stream) => {
                    let state = state.clone();
                    let token = token.clone();
                    thread::spawn(move || {
                        if let Err(error) = handle_connection(stream, &state, &token) {
                            eprintln!("Automation API request failed: {error}");
                        }
                    });
                }
                Err(error) => eprintln!("Automation API accept error: {error}"),
            }
        }
    });

    Ok(())
}


fn handle_websocket(stream:TcpStream,state:&RuntimeState,token:&str)->Result<(),String>{
    stream.set_read_timeout(Some(Duration::from_millis(500))).map_err(|e|e.to_string())?;
    let mut ws=tungstenite::accept(stream).map_err(|e|format!("WebSocket handshake failed: {e}"))?;
    let auth=ws.read().map_err(|e|e.to_string())?;
    let auth_text=auth.into_text().map_err(|e|e.to_string())?;
    let value:Value=serde_json::from_str(&auth_text).map_err(|e|e.to_string())?;
    if value.get("token").and_then(Value::as_str)!=Some(token){
        let _=ws.send(tungstenite::Message::Text(json!({"type":"error","error":"unauthorized"}).to_string().into()));
        return Err("WebSocket unauthorized".into());
    }
    ws.send(tungstenite::Message::Text(json!({"type":"ready","service":"nekodroid-automation-ws"}).to_string().into())).map_err(|e|e.to_string())?;
    let mut log_subscription:Option<String>=None;
    let mut last_log=Instant::now()-Duration::from_secs(2);
    loop{
        match ws.read(){
            Ok(message)=>{
                if message.is_close(){break;}
                if !message.is_text(){continue;}
                let req:Value=serde_json::from_str(message.to_text().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
                match req.get("type").and_then(Value::as_str).unwrap_or(""){
                    "subscribe-logs"=>{log_subscription=req.get("instanceId").and_then(Value::as_str).map(str::to_string);}
                    "unsubscribe-logs"=>{log_subscription=None;}
                    "gamepad"=>{
                        let port=req.get("port").and_then(Value::as_u64).and_then(|v|u16::try_from(v).ok()).ok_or("gamepad port required")?;
                        let button=req.get("button").and_then(Value::as_str).ok_or("gamepad button required")?;
                        let key=match button{"a"=>"KEYCODE_BUTTON_A","b"=>"KEYCODE_BUTTON_B","x"=>"KEYCODE_BUTTON_X","y"=>"KEYCODE_BUTTON_Y","up"=>"KEYCODE_DPAD_UP","down"=>"KEYCODE_DPAD_DOWN","left"=>"KEYCODE_DPAD_LEFT","right"=>"KEYCODE_DPAD_RIGHT","start"=>"KEYCODE_BUTTON_START","select"=>"KEYCODE_BUTTON_SELECT",_=>return Err("unsupported gamepad button".into())};
                        let result=adb::input_keyevent(port,key.into())?;
                        ws.send(tungstenite::Message::Text(json!({"type":"gamepad-result","result":result}).to_string().into())).map_err(|e|e.to_string())?;
                    }
                    "ping"=>{ws.send(tungstenite::Message::Text(json!({"type":"pong"}).to_string().into())).map_err(|e|e.to_string())?;}
                    _=>{}
                }
            }
            Err(tungstenite::Error::Io(ref e)) if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::TimedOut)=>{}
            Err(tungstenite::Error::ConnectionClosed)=>break,
            Err(e)=>return Err(e.to_string()),
        }
        if let Some(id)=log_subscription.as_deref(){
            if last_log.elapsed()>=Duration::from_secs(1){
                let logs=runtime::read_logs(state,id)?;
                ws.send(tungstenite::Message::Text(json!({"type":"logs","instanceId":id,"logs":logs}).to_string().into())).map_err(|e|e.to_string())?;
                last_log=Instant::now();
            }
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, state: &RuntimeState, token: &str) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;

    let request = read_request(&mut stream)?;
    let (method, path) = parse_request_line(&request.headers)?;

    if method == "GET" && path == "/health" {
        return write_json(&mut stream, 200, &json!({
            "ok": true,
            "service": "nekodroid-automation-api",
            "bind": "127.0.0.1"
        }));
    }

    if !authorized(&request.headers, token) {
        return write_json(&mut stream, 401, &json!({"error":"unauthorized"}));
    }

    let result = route(method, path, &request.body, state);
    match result {
        Ok(value) => write_json(&mut stream, 200, &value),
        Err(error) => write_json(&mut stream, 400, &json!({"error": error})),
    }
}

fn route(method: &str, path: &str, body: &[u8], state: &RuntimeState) -> Result<Value, String> {
    if method == "GET" && path == "/instances" {
        return to_value(storage::load_instances(&state.data_dir)?);
    }

    if method == "GET" && path == "/ai/settings" {
        return to_value(settings::load(&state.data_dir)?);
    }

    if method == "POST" && path == "/ai/emergency-stop" {
        return to_value(settings::emergency_stop(&state.data_dir)?);
    }

    if method == "POST" {
        if let Some(id) = path.strip_prefix("/instances/").and_then(|tail| tail.strip_suffix("/start")) {
            return to_value(runtime::start_instance(state, id)?);
        }
        if let Some(id) = path.strip_prefix("/instances/").and_then(|tail| tail.strip_suffix("/stop")) {
            return to_value(runtime::stop_instance(state, id)?);
        }
    }

    let value: Value = if body.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(body).map_err(|e| format!("Invalid JSON body: {e}"))?
    };

    match (method, path) {
        ("POST", "/adb/state") => {
            to_value(adb::get_state(required_u16(&value, "port")?)?)
        }
        ("POST", "/adb/reboot") => {
            let mode = optional_string(&value, "mode");
            to_value(adb::reboot(required_u16(&value, "port")?, mode)?)
        }
        ("POST", "/apk/install") => {
            to_value(adb::install(
                required_u16(&value, "port")?,
                required_string(&value, "apkPath")?,
            )?)
        }
        ("POST", "/files/push") => {
            to_value(adb::push(
                required_u16(&value, "port")?,
                required_string(&value, "source")?,
                required_string(&value, "destination")?,
            )?)
        }
        ("POST", "/files/pull") => {
            to_value(adb::pull(
                required_u16(&value, "port")?,
                required_string(&value, "source")?,
                required_string(&value, "destination")?,
            )?)
        }
        ("POST", "/input/tap") => {
            to_value(adb::input_tap(
                required_u16(&value, "port")?,
                required_i32(&value, "x")?,
                required_i32(&value, "y")?,
            )?)
        }
        ("POST", "/input/swipe") => {
            to_value(adb::input_swipe(
                required_u16(&value, "port")?,
                required_i32(&value, "x1")?,
                required_i32(&value, "y1")?,
                required_i32(&value, "x2")?,
                required_i32(&value, "y2")?,
                required_u32(&value, "durationMs")?,
            )?)
        }
        ("POST", "/input/keyevent") => {
            to_value(adb::input_keyevent(
                required_u16(&value, "port")?,
                required_string(&value, "keycode")?,
            )?)
        }
        ("POST", "/input/text") => {
            to_value(adb::input_text(
                required_u16(&value, "port")?,
                required_string(&value, "text")?,
            )?)
        }
        ("POST", "/rotation") => {
            to_value(adb::set_orientation(
                required_u16(&value, "port")?,
                required_string(&value, "orientation")?,
            )?)
        }
        ("POST", "/sensors/battery") => {
            let level=required_u32(&value,"level")?;
            if level>100{return Err("Battery level must be 0..100".into());}
            let charging=value.get("charging").and_then(Value::as_bool).unwrap_or(false);
            to_value(sensors::battery(required_u16(&value,"port")?,level as u8,charging)?)
        }
        ("POST", "/sensors/gps") => {
            let lat=value.get("latitude").and_then(Value::as_f64).ok_or("Missing latitude")?;
            let lon=value.get("longitude").and_then(Value::as_f64).ok_or("Missing longitude")?;
            let alt=value.get("altitude").and_then(Value::as_f64).unwrap_or(0.0);
            to_value(sensors::gps(required_u16(&value,"port")?,lat,lon,alt)?)
        }
        ("POST", "/sensors/accelerometer") => {
            let port=required_u16(&value,"port")?;
            to_value(sensors::accelerometer(port,value.get("x").and_then(Value::as_f64).unwrap_or(0.0),value.get("y").and_then(Value::as_f64).unwrap_or(0.0),value.get("z").and_then(Value::as_f64).unwrap_or(0.0))?)
        }
        ("POST", "/sensors/gyroscope") => {
            let port=required_u16(&value,"port")?;
            to_value(sensors::gyroscope(port,value.get("x").and_then(Value::as_f64).unwrap_or(0.0),value.get("y").and_then(Value::as_f64).unwrap_or(0.0),value.get("z").and_then(Value::as_f64).unwrap_or(0.0))?)
        }
        ("POST", "/sensors/compass") => to_value(sensors::compass(required_u16(&value,"port")?,value.get("heading").and_then(Value::as_f64).unwrap_or(0.0))?),
        ("POST", "/sensors/light") => to_value(sensors::light(required_u16(&value,"port")?,value.get("lux").and_then(Value::as_f64).unwrap_or(0.0))?),
        ("POST", "/sensors/proximity") => to_value(sensors::proximity(required_u16(&value,"port")?,value.get("cm").and_then(Value::as_f64).unwrap_or(0.0))?),
        ("POST", "/sensors/report") => to_value(sensors::report(required_u16(&value,"port")?)?),
        ("POST", "/screenshot") => {
            to_value(adb::screenshot(
                required_u16(&value, "port")?,
                required_string(&value, "destination")?,
            )?)
        }
        _ => Err(format!("Unknown API route: {method} {path}")),
    }
}

fn to_value<T: Serialize>(value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| e.to_string())
}

struct Request {
    headers: String,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> Result<Request, String> {
    let mut data = Vec::new();
    let mut chunk = [0u8; 8192];
    let mut header_end = None;
    let mut content_length = 0usize;

    loop {
        let read = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        data.extend_from_slice(&chunk[..read]);

        if data.len() > 1_048_576 {
            return Err("HTTP request exceeds 1 MiB limit".into());
        }

        if header_end.is_none() {
            if let Some(position) = find_bytes(&data, b"\r\n\r\n") {
                header_end = Some(position + 4);
                let headers = String::from_utf8_lossy(&data[..position + 4]);
                content_length = parse_content_length(&headers).unwrap_or(0);
            }
        }

        if let Some(end) = header_end {
            if data.len() >= end + content_length {
                let headers = String::from_utf8_lossy(&data[..end]).to_string();
                let body = data[end..end + content_length].to_vec();
                return Ok(Request { headers, body });
            }
        }
    }

    Err("Incomplete HTTP request".into())
}

fn parse_request_line(headers: &str) -> Result<(&str, &str), String> {
    let line = headers.lines().next().ok_or_else(|| "Missing HTTP request line".to_string())?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or_else(|| "Missing HTTP method".to_string())?;
    let path = parts.next().ok_or_else(|| "Missing HTTP path".to_string())?;
    Ok((method, path))
}

fn parse_content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.trim().eq_ignore_ascii_case("content-length") {
            value.trim().parse::<usize>().ok()
        } else {
            None
        }
    })
}

fn authorized(headers: &str, token: &str) -> bool {
    headers.lines().any(|line| {
        let Some((name, value)) = line.split_once(':') else {
            return false;
        };
        name.trim().eq_ignore_ascii_case("authorization")
            && value.trim() == format!("Bearer {token}")
    })
}

fn write_json<T: Serialize>(stream: &mut TcpStream, status: u16, body: &T) -> Result<(), String> {
    let body = serde_json::to_vec(body).map_err(|e| e.to_string())?;
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        _ => "Error",
    };
    let headers = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",
        body.len()
    );
    stream.write_all(headers.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(&body).map_err(|e| e.to_string())?;
    Ok(())
}

fn required_string(value: &Value, name: &str) -> Result<String, String> {
    value.get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("Missing string field: {name}"))
}

fn optional_string(value: &Value, name: &str) -> Option<String> {
    value.get(name).and_then(Value::as_str).map(str::to_string)
}

fn required_u16(value: &Value, name: &str) -> Result<u16, String> {
    let value = value.get(name).and_then(Value::as_u64)
        .ok_or_else(|| format!("Missing integer field: {name}"))?;
    u16::try_from(value).map_err(|_| format!("{name} is outside u16 range"))
}

fn required_u32(value: &Value, name: &str) -> Result<u32, String> {
    let value = value.get(name).and_then(Value::as_u64)
        .ok_or_else(|| format!("Missing integer field: {name}"))?;
    u32::try_from(value).map_err(|_| format!("{name} is outside u32 range"))
}

fn required_i32(value: &Value, name: &str) -> Result<i32, String> {
    let value = value.get(name).and_then(Value::as_i64)
        .ok_or_else(|| format!("Missing integer field: {name}"))?;
    i32::try_from(value).map_err(|_| format!("{name} is outside i32 range"))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}
