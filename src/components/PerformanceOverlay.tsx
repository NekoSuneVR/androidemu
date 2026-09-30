import { useEffect,useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GraphicsSettings,PerformanceSettings,PerformanceTelemetry } from "../types";

export default function PerformanceOverlay(){
  const [settings,setSettings]=useState<PerformanceSettings|null>(null);
  const [telemetry,setTelemetry]=useState<PerformanceTelemetry|null>(null);
  const [graphics,setGraphics]=useState<GraphicsSettings|null>(null);
  const [fps,setFps]=useState(0);
  const [frameTimes,setFrameTimes]=useState<number[]>([]);
  useEffect(()=>{
    invoke<PerformanceSettings>("get_performance_settings").then(setSettings).catch(()=>{});
    invoke<GraphicsSettings>("get_graphics_settings").then(setGraphics).catch(()=>{});
  },[]);
  useEffect(()=>{
    if(!settings||(!settings.cpuOverlay&&!settings.gpuOverlay&&!settings.ramOverlay))return;
    const refresh=()=>invoke<PerformanceTelemetry>("get_performance_telemetry").then(setTelemetry).catch(()=>{});
    refresh();const timer=window.setInterval(refresh,1000);return()=>window.clearInterval(timer);
  },[settings?.cpuOverlay,settings?.gpuOverlay,settings?.ramOverlay]);
  useEffect(()=>{
    if(!graphics?.fpsOverlay&&!graphics?.frameTimeGraph)return;
    let raf=0,last=performance.now(),frames=0,windowStart=last;
    const tick=(now:number)=>{
      const dt=now-last;last=now;frames++;
      setFrameTimes(current=>[...current.slice(-29),dt]);
      if(now-windowStart>=1000){setFps(Math.round(frames*1000/(now-windowStart)));frames=0;windowStart=now;}
      raf=requestAnimationFrame(tick);
    };
    raf=requestAnimationFrame(tick);return()=>cancelAnimationFrame(raf);
  },[graphics?.fpsOverlay,graphics?.frameTimeGraph]);
  const showPerf=Boolean(settings&&(settings.cpuOverlay||settings.gpuOverlay||settings.ramOverlay));
  const showFrames=Boolean(graphics&&(graphics.fpsOverlay||graphics.frameTimeGraph));
  if((!showPerf&&!showFrames)||(showPerf&&!telemetry))return null;
  return <div style={{position:"fixed",right:12,top:12,zIndex:9999,maxWidth:360,padding:10,border:"1px solid currentColor",borderRadius:10,background:"rgba(0,0,0,.82)",fontFamily:"monospace",fontSize:12,pointerEvents:"none"}}>
    {graphics?.fpsOverlay&&<div>UI FPS: {fps}</div>}
    {graphics?.frameTimeGraph&&<div>Frame ms: {frameTimes.map(v=>Math.round(v)).join(" · ")}</div>}
    {settings?.cpuOverlay&&<div>CPU: {telemetry?.cpu}</div>}
    {settings?.gpuOverlay&&<div>GPU: {telemetry?.gpu}</div>}
    {settings?.ramOverlay&&<div>RAM: {telemetry?.ram}</div>}
  </div>;
}
