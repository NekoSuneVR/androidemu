import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  AndroidInstance,
  CreateInstanceRequest,
  HostCapabilities,
  RuntimeActionResult
} from "./types";

const nav = ["Home", "Instances", "Device Profiles", "Media Tools", "Developer Tools", "NekoAI", "Settings"];

const defaultRequest: CreateInstanceRequest = {
  name: "Gaming",
  androidVersion: "16",
  profile: "Gaming Phone",
  cpuCores: 4,
  ramMb: 4096,
  adbPort: 5555,
  rootMode: "standard",
  imagePath: ""
};

export default function App() {
  const [instances, setInstances] = useState<AndroidInstance[]>([]);
  const [host, setHost] = useState<HostCapabilities | null>(null);
  const [active, setActive] = useState("Home");
  const [backendOnline, setBackendOnline] = useState(false);
  const [showCreate, setShowCreate] = useState(false);
  const [request, setRequest] = useState<CreateInstanceRequest>(defaultRequest);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string>("");

  const refresh = async () => {
    const [instanceData, hostData] = await Promise.all([
      invoke<AndroidInstance[]>("list_instances"),
      invoke<HostCapabilities>("get_host_capabilities")
    ]);
    setInstances(instanceData);
    setHost(hostData);
    setBackendOnline(true);
  };

  useEffect(() => {
    refresh().catch((error) => {
      setBackendOnline(false);
      setNotice(String(error));
    });

    const timer = window.setInterval(() => {
      invoke<AndroidInstance[]>("list_instances")
        .then(setInstances)
        .catch(() => setBackendOnline(false));
    }, 3000);

    return () => window.clearInterval(timer);
  }, []);

  const createInstance = async (event: FormEvent) => {
    event.preventDefault();
    try {
      const created = await invoke<AndroidInstance>("create_instance", { request });
      setInstances(current => [...current, created]);
      setShowCreate(false);
      setNotice(`${created.name} created. Add a bootable Android QCOW2 image before starting it.`);
      setRequest({ ...defaultRequest, adbPort: defaultRequest.adbPort + instances.length + 1 });
    } catch (error) {
      setNotice(String(error));
    }
  };

  const runtimeAction = async (id: string, action: "start_instance" | "stop_instance") => {
    setBusyId(id);
    try {
      const result = await invoke<RuntimeActionResult>(action, { id });
      setNotice(result.message);
      await refresh();
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusyId(null);
    }
  };

  const deleteInstance = async (instance: AndroidInstance) => {
    if (!window.confirm(`Delete ${instance.name}? Its instance configuration, logs and snapshots will be removed.`)) {
      return;
    }
    setBusyId(instance.id);
    try {
      await invoke("delete_instance", { id: instance.id });
      setInstances(current => current.filter(item => item.id !== instance.id));
      setNotice(`${instance.name} deleted.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusyId(null);
    }
  };

  const qemuStatus = host?.qemu.found
    ? host.qemu.version ?? "QEMU detected"
    : "QEMU not found";

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
          <button className="primary" onClick={() => setShowCreate(true)}>+ New Instance</button>
        </header>

        {notice && (
          <div className="notice">
            <span>{notice}</span>
            <button onClick={() => setNotice("")}>×</button>
          </div>
        )}

        <section className="hero">
          <div>
            <span className="pill">Runtime foundation</span>
            <h2>Android instance control is now wired to a native Rust runtime manager.</h2>
            <p>NekoDroid can persist instance configurations, detect QEMU and host acceleration, launch a configured QCOW2 guest, track its process, and stop it from the launcher.</p>
          </div>
          <div className="host-card">
            <span>Detected host</span>
            <strong>{host ? `${host.os} / ${host.arch}` : "Detecting..."}</strong>
            <small>{host?.accelerator ?? "Waiting for backend"} · {host?.acceleratorAvailable ? "available" : "unavailable"}</small>
            <small className="runtime-detail">{qemuStatus}</small>
          </div>
        </section>

        <section className="stats">
          <article><span>Instances</span><strong>{instances.length}</strong><small>Persistent Android environments</small></article>
          <article><span>QEMU</span><strong>{host?.qemu.found ? "Ready" : "Missing"}</strong><small>{host?.qemu.executable ?? "Install qemu-system-x86_64"}</small></article>
          <article><span>Acceleration</span><strong>{host?.accelerator ?? "Unknown"}</strong><small>{host?.virtualizationNote ?? "Capability scan pending"}</small></article>
          <article><span>Security</span><strong>Locked</strong><small>ADB/root remain opt-in per instance</small></article>
        </section>

        <section className="panel">
          <div className="panel-heading">
            <div><p className="eyebrow">Instances</p><h3>Android environments</h3></div>
            <button className="ghost" onClick={() => refresh().catch(error => setNotice(String(error)))}>Refresh</button>
          </div>
          {instances.length === 0 ? (
            <div className="empty">
              <div className="phone-outline"><div /></div>
              <h4>No Android instance yet</h4>
              <p>Create an instance now. A bootable Android x86_64 QCOW2 image can be assigned in the creation form.</p>
              <button className="primary" onClick={() => setShowCreate(true)}>Create first instance</button>
            </div>
          ) : (
            <div className="instance-grid">
              {instances.map(instance => (
                <article className="instance-card" key={instance.id}>
                  <div className="instance-title">
                    <strong>{instance.name}</strong>
                    <span className={`status status-${instance.status}`}>{instance.status}</span>
                  </div>
                  <p>Android {instance.androidVersion} · {instance.profile}</p>
                  <small>{instance.cpuCores} vCPU · {Math.round(instance.ramMb / 1024)} GB RAM · ADB localhost:{instance.adbPort}</small>
                  <small className="image-path">{instance.imagePath || "No boot image configured"}</small>
                  {instance.processId && <small>PID {instance.processId}</small>}
                  <div className="instance-actions">
                    {instance.status === "running" ? (
                      <button className="danger" disabled={busyId === instance.id} onClick={() => runtimeAction(instance.id, "stop_instance")}>Stop</button>
                    ) : (
                      <button className="primary compact" disabled={busyId === instance.id} onClick={() => runtimeAction(instance.id, "start_instance")}>Start</button>
                    )}
                    <button className="ghost compact" disabled={busyId === instance.id || instance.status === "running"} onClick={() => deleteInstance(instance)}>Delete</button>
                  </div>
                </article>
              ))}
            </div>
          )}
        </section>
      </main>

      {showCreate && (
        <div className="modal-backdrop" onMouseDown={() => setShowCreate(false)}>
          <form className="modal" onSubmit={createInstance} onMouseDown={event => event.stopPropagation()}>
            <div className="modal-heading">
              <div><p className="eyebrow">Instance Manager</p><h3>Create Android instance</h3></div>
              <button type="button" className="icon-button" onClick={() => setShowCreate(false)}>×</button>
            </div>
            <div className="form-grid">
              <label>Name<input value={request.name} onChange={e => setRequest({...request, name:e.target.value})} /></label>
              <label>Android version
                <select value={request.androidVersion} onChange={e => setRequest({...request, androidVersion:e.target.value})}>
                  {["16","15","14","13","12","11","10","9"].map(v => <option key={v} value={v}>Android {v}</option>)}
                </select>
              </label>
              <label>Profile
                <select value={request.profile} onChange={e => setRequest({...request, profile:e.target.value})}>
                  <option>Gaming Phone</option><option>Phone</option><option>Tablet</option><option>Large Tablet</option><option>Foldable</option>
                </select>
              </label>
              <label>CPU cores<input type="number" min="1" max="64" value={request.cpuCores} onChange={e => setRequest({...request, cpuCores:Number(e.target.value)})} /></label>
              <label>RAM (MB)<input type="number" min="512" step="512" value={request.ramMb} onChange={e => setRequest({...request, ramMb:Number(e.target.value)})} /></label>
              <label>ADB localhost port<input type="number" min="1" max="65535" value={request.adbPort} onChange={e => setRequest({...request, adbPort:Number(e.target.value)})} /></label>
              <label>Privilege mode
                <select value={request.rootMode} onChange={e => setRequest({...request, rootMode:e.target.value as CreateInstanceRequest["rootMode"]})}>
                  <option value="standard">Standard</option><option value="developer">Developer</option><option value="adb-root">ADB Root</option><option value="full-root">Full Root</option>
                </select>
              </label>
              <label className="wide">Android QCOW2 image path
                <input placeholder="C:\\NekoDroid\\images\\android16.qcow2 or /opt/nekodroid/images/android16.qcow2" value={request.imagePath ?? ""} onChange={e => setRequest({...request, imagePath:e.target.value})} />
              </label>
            </div>
            <div className="warning-box">
              NekoDroid does not falsify Play Integrity or hardware-backed attestation. Root and developer modes may change app compatibility.
            </div>
            <div className="modal-actions">
              <button type="button" className="ghost" onClick={() => setShowCreate(false)}>Cancel</button>
              <button className="primary" type="submit">Create Instance</button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}
