import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AndroidInstance, HostCapabilities } from "./types";

const nav = ["Home", "Instances", "Device Profiles", "Media Tools", "Developer Tools", "NekoAI", "Settings"];

export default function App() {
  const [instances, setInstances] = useState<AndroidInstance[]>([]);
  const [host, setHost] = useState<HostCapabilities | null>(null);
  const [active, setActive] = useState("Home");
  const [backendOnline, setBackendOnline] = useState(false);

  useEffect(() => {
    Promise.all([
      invoke<AndroidInstance[]>("list_instances"),
      invoke<HostCapabilities>("get_host_capabilities")
    ]).then(([instanceData, hostData]) => {
      setInstances(instanceData);
      setHost(hostData);
      setBackendOnline(true);
    }).catch(() => setBackendOnline(false));
  }, []);

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">N</div>
          <div><strong>NekoDroid</strong><span>Android runtime</span></div>
        </div>
        <nav>
          {nav.map(item => (
            <button key={item} className={active === item ? "nav-item active" : "nav-item"} onClick={() => setActive(item)}>
              {item}
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <span className={backendOnline ? "dot online" : "dot"} />
          Rust backend {backendOnline ? "online" : "offline"}
        </div>
      </aside>

      <main>
        <header className="topbar">
          <div>
            <p className="eyebrow">NekoDroid Control Center</p>
            <h1>{active}</h1>
          </div>
          <button className="primary">+ New Instance</button>
        </header>

        <section className="hero">
          <div>
            <span className="pill">Foundation build</span>
            <h2>Fast Android instances without hiding what the runtime really is.</h2>
            <p>Launcher and Rust control plane are now separated from the future QEMU/KVM/WHPX runtime, so device management, ADB, AI and media features can grow cleanly.</p>
          </div>
          <div className="host-card">
            <span>Detected host</span>
            <strong>{host ? `${host.os} / ${host.arch}` : "Detecting..."}</strong>
            <small>{host?.accelerator ?? "Waiting for backend"}</small>
          </div>
        </section>

        <section className="stats">
          <article><span>Instances</span><strong>{instances.length}</strong><small>Configured Android environments</small></article>
          <article><span>ADB</span><strong>Off</strong><small>Secure default</small></article>
          <article><span>Root</span><strong>Off</strong><small>Per-instance privilege mode</small></article>
          <article><span>Runtime</span><strong>{host?.accelerator ?? "Unknown"}</strong><small>{host?.virtualizationNote ?? "Capability scan pending"}</small></article>
        </section>

        <section className="panel">
          <div className="panel-heading">
            <div><p className="eyebrow">Instances</p><h3>Android environments</h3></div>
            <button className="ghost">Manage</button>
          </div>
          {instances.length === 0 ? (
            <div className="empty">
              <div className="phone-outline"><div /></div>
              <h4>No Android instance yet</h4>
              <p>Create the first instance after an Android image and QEMU runtime are installed.</p>
              <button className="primary">Create first instance</button>
            </div>
          ) : (
            <div className="instance-grid">
              {instances.map(instance => (
                <article className="instance-card" key={instance.id}>
                  <div className="instance-title"><strong>{instance.name}</strong><span>{instance.status}</span></div>
                  <p>Android {instance.androidVersion} · {instance.profile}</p>
                  <small>{instance.cpuCores} vCPU · {Math.round(instance.ramMb / 1024)} GB RAM · ADB {instance.adbPort}</small>
                </article>
              ))}
            </div>
          )}
        </section>
      </main>
    </div>
  );
}
