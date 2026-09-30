import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AndroidInstance } from "../types";

export default function ControllerPanel({instances}:{instances:AndroidInstance[]}) {
  const [instanceId,setInstanceId]=useState(instances[0]?.id??"");
  const [pads,setPads]=useState<{index:number;id:string;mapping:string}[]>([]);
  const [enabled,setEnabled]=useState(false);
  const [status,setStatus]=useState("Controller forwarding is off.");
  const last=useRef<Map<string,boolean>>(new Map());
  const selected=useMemo(()=>instances.find(i=>i.id===instanceId)??instances[0],[instances,instanceId]);

  useEffect(()=>{if(!instanceId&&instances[0])setInstanceId(instances[0].id);},[instances,instanceId]);
  useEffect(()=>{
    const scan=()=>setPads([...navigator.getGamepads()].filter(Boolean).map(p=>({index:p!.index,id:p!.id,mapping:p!.mapping})));
    scan(); window.addEventListener("gamepadconnected",scan);window.addEventListener("gamepaddisconnected",scan);
    const t=window.setInterval(scan,1500);return()=>{window.clearInterval(t);window.removeEventListener("gamepadconnected",scan);window.removeEventListener("gamepaddisconnected",scan);};
  },[]);
  useEffect(()=>{
    if(!enabled||!selected)return;
    let raf=0;
    const map:Record<number,string>={0:"KEYCODE_BUTTON_A",1:"KEYCODE_BUTTON_B",2:"KEYCODE_BUTTON_X",3:"KEYCODE_BUTTON_Y",4:"KEYCODE_BUTTON_L1",5:"KEYCODE_BUTTON_R1",6:"KEYCODE_BUTTON_L2",7:"KEYCODE_BUTTON_R2",8:"KEYCODE_BUTTON_SELECT",9:"KEYCODE_BUTTON_START",12:"KEYCODE_DPAD_UP",13:"KEYCODE_DPAD_DOWN",14:"KEYCODE_DPAD_LEFT",15:"KEYCODE_DPAD_RIGHT"};
    const tick=()=>{
      const pad=[...navigator.getGamepads()].find(Boolean);
      if(pad){
        Object.entries(map).forEach(([idx,key])=>{
          const pressed=Boolean(pad.buttons[Number(idx)]?.pressed);
          const k=`${pad.index}:${idx}`;
          if(pressed&&!last.current.get(k)) invoke("adb_input_keyevent",{port:selected.adbPort,keycode:key}).catch(()=>{});
          last.current.set(k,pressed);
        });
        const [lx=0,ly=0]=pad.axes;
        if(Math.abs(lx)>.55||Math.abs(ly)>.55){
          const key=Math.abs(lx)>Math.abs(ly)?(lx<0?"KEYCODE_DPAD_LEFT":"KEYCODE_DPAD_RIGHT"):(ly<0?"KEYCODE_DPAD_UP":"KEYCODE_DPAD_DOWN");
          const k=`axis:${key}`; if(!last.current.get(k)){invoke("adb_input_keyevent",{port:selected.adbPort,keycode:key}).catch(()=>{});last.current.set(k,true);setTimeout(()=>last.current.set(k,false),160);}
        }
      }
      raf=requestAnimationFrame(tick);
    };
    raf=requestAnimationFrame(tick);return()=>cancelAnimationFrame(raf);
  },[enabled,selected?.id]);

  const vibrate=async()=>{
    const pad=[...navigator.getGamepads()].find(Boolean) as any;
    try{
      if(pad?.vibrationActuator?.playEffect){await pad.vibrationActuator.playEffect("dual-rumble",{duration:350,strongMagnitude:1,weakMagnitude:.6});setStatus("Controller vibration test sent.");}
      else setStatus("This browser/controller does not expose vibration.");
    }catch(e){setStatus(String(e));}
  };

  return <section className="panel">
    <div className="panel-heading"><div><p className="eyebrow">Controllers</p><h3>Gamepad forwarding</h3></div><span className="pill">{pads.length} connected</span></div>
    <div className="tool-column">
      <label>Instance<select value={selected?.id??""} onChange={e=>setInstanceId(e.target.value)}>{instances.map(i=><option key={i.id} value={i.id}>{i.name}</option>)}</select></label>
      {pads.map(p=><article className="instance-card" key={p.index}><strong>{p.id}</strong><small>{p.mapping||"generic mapping"} · index {p.index}</small></article>)}
      <label className="checkbox-line"><input type="checkbox" checked={enabled} onChange={e=>{setEnabled(e.target.checked);setStatus(e.target.checked?"Controller forwarding enabled.":"Controller forwarding disabled.");}}/>Forward controller buttons/sticks to Android</label>
      <button className="ghost compact" onClick={vibrate}>Test vibration</button>
      <small className="muted">Uses the browser Gamepad API, so Xbox, PlayStation and generic controllers supported by the host browser/WebView can be forwarded as Android gamepad key events, including shoulder triggers. Per-game controller/touch mappings can be linked through the Keymaps and Game Library pages.</small>
      <pre className="inline-output">{status}</pre>
    </div>
  </section>;
}
