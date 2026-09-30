import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AdbResult, AndroidInstance, KeyBinding, KeymapProfile } from "../types";

const blank: KeymapProfile = {
  schemaVersion: 1,
  id: "default-game",
  name: "Default Game",
  packageName: "",
  instanceId: null,
  overlayVisible: true,
  bindings: [
    { input: "W", action: "joystick-up", x: 220, y: 1800, durationMs: 500 },
    { input: "A", action: "joystick-left", x: 220, y: 1800, durationMs: 500 },
    { input: "S", action: "joystick-down", x: 220, y: 1800, durationMs: 500 },
    { input: "D", action: "joystick-right", x: 220, y: 1800, durationMs: 500 }
  ]
};

export default function KeymapManager({ instances }: { instances: AndroidInstance[] }) {
  const [profiles,setProfiles]=useState<KeymapProfile[]>([]);
  const [profile,setProfile]=useState<KeymapProfile>(blank);
  const [instanceId,setInstanceId]=useState(instances[0]?.id ?? "");
  const [path,setPath]=useState("");
  const [output,setOutput]=useState("");
  const [busy,setBusy]=useState(false);
  const [overlayVisible,setOverlayVisible]=useState(true);
  const [mouseLook,setMouseLook]=useState(false);
  const overlayRef=useRef<HTMLDivElement|null>(null);
  const selected=useMemo(()=>instances.find(x=>x.id===instanceId)??instances[0],[instances,instanceId]);

  const refresh=()=>invoke<KeymapProfile[]>("list_keymaps").then(setProfiles).catch(e=>setOutput(String(e)));
  useEffect(()=>{refresh();},[]);
  useEffect(()=>{if(!instanceId&&instances[0])setInstanceId(instances[0].id);},[instances,instanceId]);
  useEffect(()=>{
    const onKey=(event:KeyboardEvent)=>{if(event.key==="F10"){event.preventDefault();setOverlayVisible(v=>!v);}};
    window.addEventListener("keydown",onKey);return()=>window.removeEventListener("keydown",onKey);
  },[]);
  useEffect(()=>{
    const node=overlayRef.current;if(!node||!mouseLook||!selected)return;
    const onMove=(event:MouseEvent)=>{
      if(document.pointerLockElement!==node)return;
      const dx=Math.max(-300,Math.min(300,event.movementX));
      const dy=Math.max(-300,Math.min(300,event.movementY));
      if(Math.abs(dx)<2&&Math.abs(dy)<2)return;
      const cx=540,cy=1200;
      invoke("adb_input_swipe",{port:selected.adbPort,x1:cx,y1:cy,x2:cx+dx,y2:cy+dy,durationMs:80}).catch(()=>{});
    };
    document.addEventListener("mousemove",onMove);return()=>document.removeEventListener("mousemove",onMove);
  },[mouseLook,selected?.id]);

  const save=async()=>{setBusy(true);try{const scoped={...profile,instanceId:selected?.id??null};const p=await invoke<KeymapProfile>("save_keymap",{profile:scoped});setProfile(p);await refresh();setOutput(`Saved ${p.name}`);}catch(e){setOutput(String(e));}finally{setBusy(false);}};
  const remove=async(id:string)=>{if(!confirm("Delete this keymap?"))return;setBusy(true);try{await invoke("remove_keymap",{id});await refresh();setOutput("Keymap deleted.");}catch(e){setOutput(String(e));}finally{setBusy(false);}};
  const test=async(binding:KeyBinding)=>{if(!selected)return;setBusy(true);try{const r=await invoke<AdbResult>("execute_key_binding",{port:selected.adbPort,binding});setOutput([r.success?"SUCCESS":"FAILED",r.stdout,r.stderr].filter(Boolean).join("\n"));}catch(e){setOutput(String(e));}finally{setBusy(false);}};

  const updateBinding=(index:number,patch:Partial<KeyBinding>)=>{
    const next=[...profile.bindings];next[index]={...next[index],...patch};setProfile({...profile,bindings:next});
  };
  const addBinding=()=>setProfile({...profile,bindings:[...profile.bindings,{input:"E",action:"tap",x:540,y:1200}]});

  return <section className="panel">
    <div className="panel-heading"><div><p className="eyebrow">Keymaps</p><h3>Per-game virtual controls</h3></div><span className="pill">{profiles.length} saved</span></div>
    <div className="developer-grid">
      <div className="tool-column">
        <label>Instance<select value={selected?.id??""} onChange={e=>setInstanceId(e.target.value)}>{instances.map(i=><option key={i.id} value={i.id}>{i.name}</option>)}</select></label>
        <label>Keymap ID<input value={profile.id} onChange={e=>setProfile({...profile,id:e.target.value})}/></label>
        <label>Name<input value={profile.name} onChange={e=>setProfile({...profile,name:e.target.value})}/></label>
        <label>Android package<input placeholder="com.example.game" value={profile.packageName} onChange={e=>setProfile({...profile,packageName:e.target.value})}/></label>
        <small className="muted">Saved to instance: {selected?.name ?? "none"}</small>
        <label className="checkbox-line"><input type="checkbox" checked={profile.overlayVisible} onChange={e=>setProfile({...profile,overlayVisible:e.target.checked})}/>Show touch/control overlay</label>
        {profile.bindings.map((b,index)=><article className="instance-card" key={index}>
          <div className="split-fields">
            <label>Input<input value={b.input} onChange={e=>updateBinding(index,{input:e.target.value})}/></label>
            <label>Action<select value={b.action} onChange={e=>updateBinding(index,{action:e.target.value as KeyBinding["action"]})}>
              {["tap","hold","swipe","key","text","mouse-left","mouse-right","scroll-up","scroll-down","joystick-up","joystick-down","joystick-left","joystick-right"].map(a=><option key={a}>{a}</option>)}
            </select></label>
          </div>
          <div className="split-fields">
            <label>X<input type="number" value={b.x??0} onChange={e=>updateBinding(index,{x:Number(e.target.value)})}/></label>
            <label>Y<input type="number" value={b.y??0} onChange={e=>updateBinding(index,{y:Number(e.target.value)})}/></label>
          </div>
          <div className="split-fields">
            <label>X2<input type="number" value={b.x2??0} onChange={e=>updateBinding(index,{x2:Number(e.target.value)})}/></label>
            <label>Y2<input type="number" value={b.y2??0} onChange={e=>updateBinding(index,{y2:Number(e.target.value)})}/></label>
          </div>
          <div className="split-fields">
            <label>Duration ms<input type="number" value={b.durationMs??300} onChange={e=>updateBinding(index,{durationMs:Number(e.target.value)})}/></label>
            <label>Keycode<input value={b.keycode??""} onChange={e=>updateBinding(index,{keycode:e.target.value})}/></label>
          </div>
          <div className="button-row">
            <button className="ghost compact" disabled={busy||!selected} onClick={()=>test(b)}>Test</button>
            <button className="danger compact" onClick={()=>setProfile({...profile,bindings:profile.bindings.filter((_,i)=>i!==index)})}>Remove</button>
          </div>
        </article>)}
        <div className="button-row"><button className="ghost compact" onClick={addBinding}>Add binding</button><button className="primary compact" disabled={busy} onClick={save}>Save keymap</button></div>
        <div className="tool-group">
          <h4>Visual overlay editor</h4>
          <div className="button-row">
            <button type="button" className="ghost compact" onClick={()=>setOverlayVisible(v=>!v)}>{overlayVisible?"Hide":"Show"} overlay (F10)</button>
            <button type="button" className={mouseLook?"danger compact":"ghost compact"} onClick={async()=>{const next=!mouseLook;setMouseLook(next);if(next)await overlayRef.current?.requestPointerLock();else if(document.pointerLockElement)document.exitPointerLock();}}>{mouseLook?"Release mouse":"Mouse-look / lock"}</button>
          </div>
          <div ref={overlayRef} style={{position:"relative",width:"100%",aspectRatio:"9 / 16",maxHeight:520,border:"1px solid currentColor",overflow:"hidden",cursor:mouseLook?"crosshair":"default"}}>
            {overlayVisible&&profile.bindings.filter(b=>b.x!=null&&b.y!=null).map((b,index)=><button
              type="button"
              key={index}
              title={`${b.input} → ${b.action}`}
              style={{position:"absolute",left:`${Math.max(0,Math.min(100,(Number(b.x)/1080)*100))}%`,top:`${Math.max(0,Math.min(100,(Number(b.y)/2400)*100))}%`,transform:"translate(-50%,-50%)"}}
              onPointerDown={event=>{if(mouseLook)return;const rect=(event.currentTarget.parentElement as HTMLElement).getBoundingClientRect();const move=(ev:PointerEvent)=>{const x=Math.round(Math.max(0,Math.min(1,(ev.clientX-rect.left)/rect.width))*1080);const y=Math.round(Math.max(0,Math.min(1,(ev.clientY-rect.top)/rect.height))*2400);updateBinding(index,{x,y});};const up=()=>{window.removeEventListener("pointermove",move);window.removeEventListener("pointerup",up);};window.addEventListener("pointermove",move);window.addEventListener("pointerup",up);}}
            >{b.input}</button>)}
          </div>
          <small className="muted">Drag overlay markers to edit touch positions. F10 shows/hides controls. Mouse-look uses pointer lock and forwards relative motion as short Android swipes.</small>
        </div>
        <label>Import/export path<input value={path} onChange={e=>setPath(e.target.value)}/></label>
        <div className="button-row">
          <button className="ghost compact" disabled={!path} onClick={async()=>{try{const p=await invoke<KeymapProfile>("import_keymap",{source:path});setProfile(p);await refresh();setOutput("Imported.");}catch(e){setOutput(String(e));}}}>Import</button>
          <button className="ghost compact" disabled={!path||!profile.id} onClick={async()=>{try{await invoke("export_keymap",{id:profile.id,destination:path});setOutput("Exported.");}catch(e){setOutput(String(e));}}}>Export</button>
        </div>
      </div>
      <div className="terminal"><div className="terminal-title">Saved keymaps</div>{profiles.map(p=><div key={p.id} className="button-row"><button className="ghost compact" onClick={()=>setProfile(p)}>{p.name}</button><button className="danger compact" onClick={()=>remove(p.id)}>Delete</button></div>)}<pre>{output}</pre></div>
    </div>
  </section>;
}
