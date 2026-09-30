import { useMemo,useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AdbResult,AndroidInstance } from "../types";

export default function SensorPanel({instances}:{instances:AndroidInstance[]}) {
  const [instanceId,setInstanceId]=useState(instances[0]?.id??"");
  const [battery,setBattery]=useState(100); const [charging,setCharging]=useState(true);
  const [lat,setLat]=useState(51.5074); const [lon,setLon]=useState(-0.1278); const [alt,setAlt]=useState(20);
  const [accel,setAccel]=useState({x:0,y:9.81,z:0});
  const [gyro,setGyro]=useState({x:0,y:0,z:0});
  const [heading,setHeading]=useState(0); const [lux,setLux]=useState(100); const [proximity,setProximity]=useState(5);
  const [out,setOut]=useState("");
  const selected=useMemo(()=>instances.find(i=>i.id===instanceId)??instances[0],[instances,instanceId]);
  const run=async(name:string,args:Record<string,unknown>)=>{if(!selected)return;try{const r=await invoke<AdbResult>(name,{port:selected.adbPort,...args});setOut([r.success?"SUCCESS":"FAILED",r.stdout,r.stderr].filter(Boolean).join("\n"));}catch(e){setOut(String(e));}};
  return <section className="panel">
    <div className="panel-heading"><div><p className="eyebrow">Virtual Sensors</p><h3>Battery and location simulation</h3></div></div>
    <div className="developer-grid"><div className="tool-column">
      <label>Instance<select value={selected?.id??""} onChange={e=>setInstanceId(e.target.value)}>{instances.map(i=><option key={i.id} value={i.id}>{i.name}</option>)}</select></label>
      <div className="tool-group"><h4>Battery</h4>
        <label>Level %<input type="number" min="0" max="100" value={battery} onChange={e=>setBattery(Number(e.target.value))}/></label>
        <label className="checkbox-line"><input type="checkbox" checked={charging} onChange={e=>setCharging(e.target.checked)}/>Charging</label>
        <div className="button-row"><button className="primary compact" onClick={()=>run("set_battery_simulation",{level:battery,charging})}>Apply battery</button><button className="ghost compact" onClick={()=>run("reset_battery_simulation",{})}>Reset battery</button></div>
      </div>
      <div className="tool-group"><h4>GPS test location</h4>
        <div className="split-fields"><label>Latitude<input type="number" step="0.000001" value={lat} onChange={e=>setLat(Number(e.target.value))}/></label><label>Longitude<input type="number" step="0.000001" value={lon} onChange={e=>setLon(Number(e.target.value))}/></label><label>Altitude<input type="number" value={alt} onChange={e=>setAlt(Number(e.target.value))}/></label></div>
        <div className="button-row"><button className="primary compact" onClick={()=>run("set_gps_location",{latitude:lat,longitude:lon,altitude:alt})}>Set test location</button><button className="ghost compact" onClick={()=>run("clear_gps_location",{})}>Clear test provider</button></div>
      </div>
      <div className="tool-group"><h4>Motion / environment bridge</h4>
        <div className="split-fields">
          <label>Accel X<input type="number" step="0.01" value={accel.x} onChange={e=>setAccel({...accel,x:Number(e.target.value)})}/></label>
          <label>Accel Y<input type="number" step="0.01" value={accel.y} onChange={e=>setAccel({...accel,y:Number(e.target.value)})}/></label>
          <label>Accel Z<input type="number" step="0.01" value={accel.z} onChange={e=>setAccel({...accel,z:Number(e.target.value)})}/></label>
        </div>
        <button className="ghost compact" onClick={()=>run("set_accelerometer",accel)}>Set accelerometer</button>
        <div className="split-fields">
          <label>Gyro X<input type="number" step="0.01" value={gyro.x} onChange={e=>setGyro({...gyro,x:Number(e.target.value)})}/></label>
          <label>Gyro Y<input type="number" step="0.01" value={gyro.y} onChange={e=>setGyro({...gyro,y:Number(e.target.value)})}/></label>
          <label>Gyro Z<input type="number" step="0.01" value={gyro.z} onChange={e=>setGyro({...gyro,z:Number(e.target.value)})}/></label>
        </div>
        <button className="ghost compact" onClick={()=>run("set_gyroscope",gyro)}>Set gyroscope</button>
        <div className="split-fields">
          <label>Compass °<input type="number" value={heading} onChange={e=>setHeading(Number(e.target.value))}/></label>
          <label>Light lux<input type="number" value={lux} onChange={e=>setLux(Number(e.target.value))}/></label>
          <label>Proximity cm<input type="number" value={proximity} onChange={e=>setProximity(Number(e.target.value))}/></label>
        </div>
        <div className="button-row">
          <button className="ghost compact" onClick={()=>run("set_compass",{heading})}>Set compass</button>
          <button className="ghost compact" onClick={()=>run("set_light_sensor",{lux})}>Set light</button>
          <button className="ghost compact" onClick={()=>run("set_proximity_sensor",{cm:proximity})}>Set proximity</button>
        </div>
        <small className="muted">Motion/environment values use the NekoDroid guest sensor-provider protocol. Generic Android apps require an image/provider that implements that protocol.</small>
      </div>
      <button className="ghost compact" onClick={()=>run("sensor_report",{})}>Read sensor report</button>
    </div><pre className="terminal">{out}</pre></div>
  </section>;
}
