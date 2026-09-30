import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import DeviceProfiles from "./components/DeviceProfiles";
import ImageManager from "./components/ImageManager";
import RemoteAccess from "./components/RemoteAccess";
import DeveloperTools from "./components/DeveloperTools";
import NekoAI from "./components/NekoAI";
import SnapshotManager from "./components/SnapshotManager";
import MediaTools from "./components/MediaTools";
import SettingsPage from "./components/SettingsPage";
import SystemReadinessPanel from "./components/SystemReadinessPanel";
import FileManager from "./components/FileManager";
import GameLibrary from "./components/GameLibrary";
import FirstRunWizard from "./components/FirstRunWizard";
import KeymapManager from "./components/KeymapManager";
import ControllerPanel from "./components/ControllerPanel";
import SensorPanel from "./components/SensorPanel";
import type {
  AdbInfo,
  AndroidInstance,
  CreateInstanceRequest,
  DeviceProfile,
  HostCapabilities,
  InstalledImage,
  RuntimeActionResult,
  AppSettings
} from "./types";

const nav = ["Home", "Game Library", "Instances", "Android Images", "Device Profiles", "File Manager", "Keymaps", "Remote Access", "Media Tools", "Developer Tools", "NekoAI", "Settings"];

const defaultRequest: CreateInstanceRequest = {
  name: "Gaming",
  androidVersion: "16",
  profile: "Gaming Phone",
  cpuCores: 4,
  ramMb: 4096,
  adbPort: 5555,
  adbEnabled: false,
  headless: false,
  rootMode: "standard",
  imagePath: ""
};

export default function App() {
  const [instances, setInstances] = useState<AndroidInstance[]>([]);
  const [profiles, setProfiles] = useState<DeviceProfile[]>([]);
  const [images, setImages] = useState<InstalledImage[]>([]);
  const [host, setHost] = useState<HostCapabilities | null>(null);
  const [adbInfo, setAdbInfo] = useState<AdbInfo | null>(null);
  const [active, setActive] = useState("Home");
  const [backendOnline, setBackendOnline] = useState(false);
  const [showCreate, setShowCreate] = useState(false);
  const [request, setRequest] = useState<CreateInstanceRequest>(defaultRequest);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string>("");

  const refresh = async () => {
    const [instanceData, hostData, profileData, adbData, imageData, appSettings] = await Promise.all([
      invoke<AndroidInstance[]>("list_instances"),
      invoke<HostCapabilities>("get_host_capabilities"),
      invoke<DeviceProfile[]>("list_device_profiles"),
      invoke<AdbInfo>("get_adb_info"),
      invoke<InstalledImage[]>("list_android_images"),
      invoke<AppSettings>("get_app_settings")
    ]);
    setInstances(instanceData);
    setHost(hostData);
    setProfiles(profileData);
    setAdbInfo(adbData);
    setImages(imageData);
    setRequest(current => ({
      ...current,
      androidVersion: appSettings.defaultAndroidVersion,
      profile: appSettings.defaultProfile,
      adbEnabled: appSettings.defaultAdbEnabled,
      headless: appSettings.defaultHeadless
    }));
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
      setNotice(`${created.name} created. Add a bootable Android x86_64 QCOW2 or raw image before starting it.`);
      setRequest({ ...defaultRequest, adbPort: defaultRequest.adbPort + instances.length + 1 });
    } catch (error) {
      setNotice(String(error));
    }
  };

  const applyProfile = (name: string) => {
    const profile = profiles.find(item => item.name === name);
    if (!profile) {
      setRequest(current => ({ ...current, profile: name }));
      return;
    }
    setRequest(current => ({
      ...current,
      profile: profile.name,
      cpuCores: profile.defaultCpuCores,
      ramMb: profile.defaultRamMb
    }));
  };

  const runtimeAction = async (
    id: string,
    action: "start_instance" | "pause_instance" | "resume_instance" | "stop_instance"
  ) => {
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

  const updateInstance = async (instance: AndroidInstance, patch: { name?: string; adbEnabled?: boolean; headless?: boolean }) => {
    setBusyId(instance.id);
    try {
      const updated = await invoke<AndroidInstance>("update_instance", {
        id: instance.id,
        request: {
          name: patch.name ?? instance.name,
          adbEnabled: patch.adbEnabled ?? instance.adbEnabled,
          headless: patch.headless ?? instance.headless
        }
      });
      setInstances(current => current.map(item => item.id === updated.id ? updated : item));
      setNotice(`${updated.name} settings updated.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusyId(null);
    }
  };

  const renameInstance = async (instance: AndroidInstance) => {
    const name = window.prompt("Rename Android instance", instance.name)?.trim();
    if (!name || name === instance.name) return;
    await updateInstance(instance, { name });
  };

  const cloneInstance = async (instance: AndroidInstance) => {
    const name = window.prompt("Clone Android instance", `${instance.name} Clone`)?.trim();
    if (!name) return;
    setBusyId(instance.id);
    try {
      const cloned = await invoke<AndroidInstance>("clone_instance", { id: instance.id, name });
      setInstances(current => [...current, cloned]);
      setNotice(`${cloned.name} cloned from ${instance.name}. ADB is disabled on the clone until you enable it.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusyId(null);
    }
  };

  const factoryResetInstance = async (instance: AndroidInstance) => {
    if (!window.confirm(`Factory reset ${instance.name}? This deletes its writable runtime disk and all snapshots. The registered base image is kept.`)) return;
    setBusyId(instance.id);
    try {
      const result = await invoke<RuntimeActionResult>("factory_reset_instance", { id: instance.id });
      setNotice(result.message);
      await refresh();
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusyId(null);
    }
  };

  const deleteInstance = async (instance: AndroidInstance) => {
    if (!window.confirm(`Delete ${instance.name}? Its instance configuration, logs and snapshots will be removed.`)) return;
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

  const qemuStatus = host?.qemu.found ? host.qemu.version ?? "QEMU detected" : "QEMU not found";

  const instancesPanel = (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Instances</p><h3>Android environments</h3></div>
        <button className="ghost" onClick={() => refresh().catch(error => setNotice(String(error)))}>Refresh</button>
      </div>
      {instances.length === 0 ? (
        <div className="empty">
          <div className="phone-outline"><div /></div>
          <h4>No Android instance yet</h4>
          <p>Create an instance now. A bootable Android x86_64 QCOW2 or raw disk can be assigned in the creation form.</p>
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
              <small>{instance.cpuCores} vCPU · {Math.round(instance.ramMb / 1024)} GB RAM · {instance.adbEnabled ? `ADB localhost:${instance.adbPort}` : "ADB disabled"} · {instance.headless ? "Headless" : "Windowed"}</small>
              <small className="image-path">{instance.imagePath || "No boot image configured"}</small>
              {instance.processId && <small>PID {instance.processId}</small>}
              <div className="instance-actions">
                <button className="ghost compact" disabled={busyId === instance.id || instance.status === "running"} onClick={() => renameInstance(instance)}>Rename</button>
                <button
                  className="ghost compact"
                  disabled={busyId === instance.id || instance.status === "running"}
                  onClick={() => updateInstance(instance, { adbEnabled: !instance.adbEnabled })}
                >
                  {instance.adbEnabled ? "Disable ADB" : "Enable ADB"}
                </button>
                <button
                  className="ghost compact"
                  disabled={busyId === instance.id || instance.status === "running"}
                  onClick={() => updateInstance(instance, { headless: !instance.headless })}
                >
                  {instance.headless ? "Use Window" : "Use Headless"}
                </button>
                {instance.status === "running" ? (
                  <>
                    <button className="ghost compact" disabled={busyId === instance.id} onClick={() => runtimeAction(instance.id, "pause_instance")}>Pause</button>
                    <button className="danger" disabled={busyId === instance.id} onClick={() => runtimeAction(instance.id, "stop_instance")}>Stop</button>
                  </>
                ) : instance.status === "paused" ? (
                  <>
                    <button className="primary compact" disabled={busyId === instance.id} onClick={() => runtimeAction(instance.id, "resume_instance")}>Resume</button>
                    <button className="danger" disabled={busyId === instance.id} onClick={() => runtimeAction(instance.id, "stop_instance")}>Stop</button>
                  </>
                ) : (
                  <button className="primary compact" disabled={busyId === instance.id} onClick={() => runtimeAction(instance.id, "start_instance")}>Start</button>
                )}
                {["adb-root","full-root"].includes(instance.rootMode) && <button className="ghost compact" disabled={busyId === instance.id || instance.status === "running"} onClick={async()=>{try{await invoke("set_root_on_next_boot",{id:instance.id,enabled:true});setNotice(`${instance.name} will request ADB root after its next boot.`);}catch(error){setNotice(String(error));}}}>Root next boot</button>}
                <button className="ghost compact" disabled={busyId === instance.id || instance.status === "running"} onClick={() => cloneInstance(instance)}>Clone</button>
                <button className="danger compact" disabled={busyId === instance.id || instance.status === "running"} onClick={() => factoryResetInstance(instance)}>Factory Reset</button>
                <button className="ghost compact" disabled={busyId === instance.id || instance.status === "running"} onClick={() => deleteInstance(instance)}>Delete</button>
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );

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

        {notice && <div className="notice"><span>{notice}</span><button onClick={() => setNotice("")}>×</button></div>}

        {active === "Home" && (
          <>
            <FirstRunWizard />
            <section className="hero">
              <div>
                <span className="pill">Runtime foundation</span>
                <h2>Native Android runtime management, ready for the first bootable image.</h2>
                <p>NekoDroid now persists instances, probes QEMU/KVM/WHPX, manages QEMU processes, exposes localhost ADB tooling, and defines realistic Android device profiles.</p>
              </div>
              <div className="host-card">
                <span>Detected host</span>
                <strong>{host ? `${host.os} / ${host.arch}` : "Detecting..."}</strong>
                <small>{host?.accelerator ?? "Waiting for backend"} · {host?.acceleratorAvailable ? "available" : "unavailable"}</small>
                <small className="runtime-detail">{qemuStatus}</small>
              </div>
            </section>

            <SystemReadinessPanel />

            <section className="stats">
              <article><span>Instances</span><strong>{instances.length}</strong><small>Persistent Android environments</small></article>
              <article><span>QEMU</span><strong>{host?.qemu.found ? "Ready" : "Missing"}</strong><small>{host?.qemu.executable ?? "Install qemu-system-x86_64"}</small></article>
              <article><span>Acceleration</span><strong>{host?.accelerator ?? "Unknown"}</strong><small>{host?.virtualizationNote ?? "Capability scan pending"}</small></article>
              <article><span>ADB</span><strong>{adbInfo?.found ? "Ready" : "Missing"}</strong><small>{adbInfo?.version ?? "Android platform-tools not detected"}</small></article>
            </section>
            {instancesPanel}
          </>
        )}

        {active === "Game Library" && <GameLibrary instances={instances} />}
        {active === "Instances" && (
          <>
            {instancesPanel}
            <SnapshotManager instances={instances} />
          </>
        )}
        {active === "Android Images" && <ImageManager images={images} onChanged={refresh} />}
        {active === "Device Profiles" && <DeviceProfiles profiles={profiles} onChanged={refresh} />}
        {active === "File Manager" && <FileManager instances={instances} />}
        {active === "Keymaps" && <KeymapManager instances={instances} />}
        {active === "Controllers" && <ControllerPanel instances={instances} />}
        {active === "Sensors" && <SensorPanel instances={instances} />}
        {active === "Remote Access" && <RemoteAccess instances={instances} profiles={profiles} />}
        {active === "Developer Tools" && <DeveloperTools instances={instances} adbInfo={adbInfo} />}
        {active === "NekoAI" && <NekoAI instances={instances} />}
        {active === "Media Tools" && <MediaTools />}
        {active === "Settings" && <SettingsPage profiles={profiles} />}
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
                <select value={request.profile} onChange={e => applyProfile(e.target.value)}>
                  {(profiles.length ? profiles.map(profile => profile.name) : ["Gaming Phone","Phone","Tablet","Large Tablet","Foldable"])
                    .map(name => <option key={name}>{name}</option>)}
                </select>
              </label>
              <label>CPU cores<input type="number" min="1" max="64" value={request.cpuCores} onChange={e => setRequest({...request, cpuCores:Number(e.target.value)})} /></label>
              <label>RAM (MB)<input type="number" min="512" step="512" value={request.ramMb} onChange={e => setRequest({...request, ramMb:Number(e.target.value)})} /></label>
              <label>ADB localhost port<input type="number" min="1" max="65535" value={request.adbPort} onChange={e => setRequest({...request, adbPort:Number(e.target.value)})} /></label>
              <label className="checkbox-line">
                <input type="checkbox" checked={request.adbEnabled} onChange={e => setRequest({...request, adbEnabled:e.target.checked})} />
                Enable localhost ADB for this instance
              </label>
              <label className="checkbox-line">
                <input type="checkbox" checked={request.headless} onChange={e => setRequest({...request, headless:e.target.checked})} />
                Start without a QEMU display window
              </label>
              <label>Privilege mode
                <select value={request.rootMode} onChange={e => setRequest({...request, rootMode:e.target.value as CreateInstanceRequest["rootMode"]})}>
                  <option value="standard">Standard</option><option value="developer">Developer</option><option value="adb-root">ADB Root</option><option value="full-root">Full Root</option>
                </select>
              </label>
              <label className="wide">Registered Android image
                <select value={images.some(image => image.diskPath === request.imagePath) ? request.imagePath ?? "" : ""} onChange={e => setRequest({...request, imagePath:e.target.value})}>
                  <option value="">Custom / none</option>
                  {images.filter(image => image.valid).map(image => (
                    <option key={image.manifest.id} value={image.diskPath}>
                      {image.manifest.name} · Android {image.manifest.androidVersion}
                    </option>
                  ))}
                </select>
              </label>
              <label className="wide">Android boot disk path
                <input placeholder="C:\\NekoDroid\\images\\android16.qcow2 or /opt/nekodroid/images/android16.img" value={request.imagePath ?? ""} onChange={e => setRequest({...request, imagePath:e.target.value})} />
              </label>
            </div>
            <div className="warning-box">
              NekoDroid does not falsify Play Integrity or hardware-backed attestation. Root/developer modes can break app compatibility and may cause Play Integrity checks to fail. Only use root with images that explicitly support it.
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
