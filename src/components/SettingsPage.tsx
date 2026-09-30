import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, DeviceProfile } from "../types";

const defaults: AppSettings = {
  defaultAndroidVersion: "16",
  defaultProfile: "Gaming Phone",
  defaultAdbEnabled: false,
  defaultHeadless: false,
  confirmDangerousActions: true
};

export default function SettingsPage({ profiles }: { profiles: DeviceProfile[] }) {
  const [settings, setSettings] = useState<AppSettings>(defaults);
  const [status, setStatus] = useState("Settings are stored locally on this PC.");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    invoke<AppSettings>("get_app_settings")
      .then(setSettings)
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

          <button className="primary" disabled={busy}>{busy ? "Saving..." : "Save Settings"}</button>

          <div className="warning-box">
            ADB remains localhost-only. Enabling ADB by default exposes a guest debugging port only to this host unless you later add an explicit LAN feature.
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
