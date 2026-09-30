import { useEffect,useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PerformanceSettings,PerformanceTelemetry } from "../types";

export default function PerformanceOverlay(){
  const [settings,setSettings]=useState<PerformanceSettings|null>(null);
  const [telemetry,setTelemetry]=useState<PerformanceTelemetry|null>(null);
  useEffect(()=>{invoke<PerformanceSettings>("get_performance_settings").then(setSettings).catch(()=>{});},[]);
  useEffect(()=>{
    if(!settings||(!settings.cpuOverlay&&!settings.gpuOverlay&&!settings.ramOverlay))return;
    const refresh=()=>invoke<PerformanceTelemetry>("get_performance_telemetry").then(setTelemetry).catch(()=>{});
    refresh();const timer=window.setInterval(refresh,1000);return()=>window.clearInterval(timer);
  },[settings?.cpuOverlay,settings?.gpuOverlay,settings?.ramOverlay]);
  if(!settings||!telemetry||(!settings.cpuOverlay&&!settings.gpuOverlay&&!settings.ramOverlay))return null;
  return <div style={{position:"fixed",right:12,top:12,zIndex:9999,maxWidth:360,padding:10,border:"1px solid currentColor",borderRadius:10,background:"rgba(0,0,0,.82)",fontFamily:"monospace",fontSize:12,pointerEvents:"none"}}>
    {settings.cpuOverlay&&<div>CPU: {telemetry.cpu}</div>}
    {settings.gpuOverlay&&<div>GPU: {telemetry.gpu}</div>}
    {settings.ramOverlay&&<div>RAM: {telemetry.ram}</div>}
  </div>;
}
