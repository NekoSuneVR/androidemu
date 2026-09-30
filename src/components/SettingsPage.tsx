import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, DeviceProfile, PerformanceSettings, UpdateCheck } from "../types";

const defaults: AppSettings = {
  defaultAndroidVersion: "16",
  defaultProfile: "Gaming Phone",
  defaultAdbEnabled: false,
  defaultHeadless: false,
  confirmDangerousActions: true,
  apiEnabled: false,
  apiPort: 37891,
  apiToken: "",
  firstRunCompleted: false
};

export default function SettingsPage({ profiles }: { profiles: DeviceProfile[] }) {
  const [settings, setSettings] = useState<AppSettings>(defaults);
  const [status, setStatus] = useState("Settings are stored locally on this PC.");
  const [busy, setBusy] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateCheck | null>(null);
  const [performance, setPerformance] = useState<PerformanceSettings | null>(null);

  useEffect(() => {
    invoke<AppSettings>("get_app_settings")
      .then(setSettings)
      .catch(error => setStatus(String(error)));
    invoke<PerformanceSettings>("get_performance_settings")
      .then(setPerformance)
      .catch(error => setStatus(String(error)));
  }, []);

  const save = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      const saved = await invoke<AppSettings>("save_app_settings", { settings });
      setSettings(saved);
      setStatus("Settings saved. New instances will use these defaults.");
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Settings</p><h3>NekoDroid defaults and safety</h3></div>
        <span className="pill">Local config</span>
      </div>

      <div className="developer-grid">
        <form className="tool-column" onSubmit={save}>
          <label>Default Android version
            <select value={settings.defaultAndroidVersion} onChange={e => setSettings({...settings, defaultAndroidVersion:e.target.value})}>
              {["16","15","14","13","12","11","10","9"].map(version => (
                <option key={version} value={version}>Android {version}</option>
              ))}
            </select>
          </label>

          <label>Default device profile
            <select value={settings.defaultProfile} onChange={e => setSettings({...settings, defaultProfile:e.target.value})}>
              {(profiles.length ? profiles : []).map(profile => (
                <option key={profile.id} value={profile.name}>{profile.name}</option>
              ))}
              {!profiles.some(profile => profile.name === settings.defaultProfile) && (
                <option value={settings.defaultProfile}>{settings.defaultProfile}</option>
              )}
            </select>
          </label>

          <label className="checkbox-line">
            <input
              type="checkbox"
              checked={settings.defaultAdbEnabled}
              onChange={e => setSettings({...settings, defaultAdbEnabled:e.target.checked})}
            />
            Enable localhost ADB by default for new instances
          </label>

          <label className="checkbox-line">
            <input
              type="checkbox"
              checked={settings.defaultHeadless}
              onChange={e => setSettings({...settings, defaultHeadless:e.target.checked})}
            />
            Start new instances in headless mode by default
          </label>

          <label className="checkbox-line">
            <input
              type="checkbox"
              checked={settings.confirmDangerousActions}
              onChange={e => setSettings({...settings, confirmDangerousActions:e.target.checked})}
            />
            Ask for confirmation before destructive actions
          </label>

          <div className="tool-group">
            <h4>Local Automation API</h4>
            <label className="checkbox-line">
              <input
                type="checkbox"
                checked={settings.apiEnabled}
                onChange={e => setSettings({...settings, apiEnabled:e.target.checked})}
              />
              Enable localhost REST API after restart
            </label>
            <label>Port
              <input type="number" min="1" max="65535" value={settings.apiPort} onChange={e => setSettings({...settings, apiPort:Number(e.target.value)})} />
            </label>
            <label>Bearer token
              <input
                type="password"
                placeholder="At least 16 characters"
                value={settings.apiToken}
                onChange={e => setSettings({...settings, apiToken:e.target.value})}
              />
            </label>
            <small className="muted">The API binds only to 127.0.0.1. Restart NekoDroid after changing API settings.</small>
          </div>

          <button className="primary" disabled={busy}>{busy ? "Saving..." : "Save Settings"}</button>

          <div className="tool-group">
            <h4>Installation repair</h4>
            <button type="button" className="ghost compact" disabled={busy} onClick={async()=>{try{setBusy(true);const actions=await invoke<string[]>("repair_installation");setStatus(["Installation repair completed.",...actions].join("\n"));}catch(error){setStatus(String(error));}finally{setBusy(false);}}}>Check & Repair Installation</button>
          </div>

          <div className="tool-group">
            <h4>Updates</h4>
            <button type="button" className="ghost compact" disabled={busy} onClick={async () => {
              setBusy(true);
              try {
                const info = await invoke<UpdateCheck>("check_for_updates");
                setUpdateInfo(info);
                setStatus(info.updateAvailable
                  ? `NekoDroid ${info.latestVersion} is available (current ${info.currentVersion}).`
                  : `NekoDroid ${info.currentVersion} is up to date.`);
              } catch (error) {
                setStatus(String(error));
              } finally {
                setBusy(false);
              }
            }}>Check GitHub Releases</button>
            {updateInfo && <small className="muted">Current {updateInfo.currentVersion} · Latest {updateInfo.latestVersion}</small>}
          </div>

          {performance && <div className="tool-group">
            <h4>Performance tuning</h4>
            <label>CPU affinity (Linux taskset syntax)<input value={performance.cpuAffinity} placeholder="0-7" onChange={e=>setPerformance({...performance,cpuAffinity:e.target.value})}/></label>
            <label className="checkbox-line"><input type="checkbox" checked={performance.hugePages} onChange={e=>setPerformance({...performance,hugePages:e.target.checked})}/>Use Linux huge pages when /dev/hugepages is available</label>
            <div className="split-fields">
              <label>I/O mode<select value={performance.ioMode} onChange={e=>setPerformance({...performance,ioMode:e.target.value as PerformanceSettings["ioMode"]})}><option value="native">native</option><option value="threads">threads</option></select></label>
              <label>Disk cache<select value={performance.diskCache} onChange={e=>setPerformance({...performance,diskCache:e.target.value as PerformanceSettings["diskCache"]})}><option value="none">none</option><option value="writeback">writeback</option><option value="writethrough">writethrough</option><option value="directsync">directsync</option></select></label>
            </div>
            <label className="checkbox-line"><input type="checkbox" checked={performance.ramCompression} onChange={e=>setPerformance({...performance,ramCompression:e.target.checked})}/>Prefer host RAM compression when available</label>
            <label className="checkbox-line"><input type="checkbox" checked={performance.shaderCache} onChange={e=>setPerformance({...performance,shaderCache:e.target.checked})}/>Enable Mesa shader cache</label>
            <div className="split-fields">
              <label>Audio latency ms<input type="number" min="10" max="250" value={performance.audioLatencyMs} onChange={e=>setPerformance({...performance,audioLatencyMs:Number(e.target.value)})}/></label>
              <label>Input latency<select value={performance.inputLatencyMode} onChange={e=>setPerformance({...performance,inputLatencyMode:e.target.value as PerformanceSettings["inputLatencyMode"]})}><option value="balanced">balanced</option><option value="low-latency">low-latency</option></select></label>
            </div>
            <label>Frame pacing<select value={performance.framePacing} onChange={e=>setPerformance({...performance,framePacing:e.target.value as PerformanceSettings["framePacing"]})}><option value="balanced">balanced</option><option value="smooth">smooth</option><option value="low-latency">low-latency</option></select></label>
            <label className="checkbox-line"><input type="checkbox" checked={performance.startupOptimization} onChange={e=>setPerformance({...performance,startupOptimization:e.target.checked})}/>Startup optimization</label>
            <label className="checkbox-line"><input type="checkbox" checked={performance.minimizeBackgroundServices} onChange={e=>setPerformance({...performance,minimizeBackgroundServices:e.target.checked})}/>Minimize guest background services</label>
            <div className="permission-grid">
              <label><input type="checkbox" checked={performance.cpuOverlay} onChange={e=>setPerformance({...performance,cpuOverlay:e.target.checked})}/>CPU overlay</label>
              <label><input type="checkbox" checked={performance.gpuOverlay} onChange={e=>setPerformance({...performance,gpuOverlay:e.target.checked})}/>GPU overlay</label>
              <label><input type="checkbox" checked={performance.ramOverlay} onChange={e=>setPerformance({...performance,ramOverlay:e.target.checked})}/>RAM overlay</label>
            </div>
            <button type="button" className="ghost compact" onClick={async()=>{try{const saved=await invoke<PerformanceSettings>("save_performance_settings",{settings:performance});setPerformance(saved);setStatus("Performance settings saved; launch-affecting changes apply to new emulator starts.");}catch(error){setStatus(String(error));}}}>Save performance settings</button>
          </div>}

          <div className="warning-box">
            ADB, QMP, and the Automation API are designed to stay localhost-only by default. Do not expose ADB, QMP, or the Automation API directly to the internet or an untrusted LAN. Use authenticated remote-access features instead.
          </div>
        </form>

        <div className="terminal">
          <div className="terminal-title">Settings status</div>
          <pre>{status}</pre>
        </div>
      </div>
    </section>
  );
}
