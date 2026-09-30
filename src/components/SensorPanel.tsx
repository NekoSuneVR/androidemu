import { useMemo,useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AdbResult,AndroidInstance } from "../types";

export default function SensorPanel({instances}:{instances:AndroidInstance[]}) {
  const [instanceId,setInstanceId]=useState(instances[0]?.id??"");
  const [battery,setBattery]=useState(100); const [charging,setCharging]=useState(true);
  const [lat,setLat]=useState(51.5074); const [lon,setLon]=useState(-0.1278); const [alt,setAlt]=useState(20);
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
      <button className="ghost compact" onClick={()=>run("sensor_report",{})}>Read sensor report</button>
    </div><pre className="terminal">{out}</pre></div>
  </section>;
}
