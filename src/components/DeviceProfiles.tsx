import { FormEvent, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DeviceProfile } from "../types";

type Props = {
  profiles: DeviceProfile[];
  onChanged: () => Promise<void>;
};

const blankProfile: DeviceProfile = {
  id: "custom-phone",
  name: "Custom Phone",
  width: 1080,
  height: 2400,
  dpi: 420,
  refreshRate: 60,
  defaultCpuCores: 4,
  defaultRamMb: 4096,
  touchPoints: 10,
  telephony: true,
  formFactor: "phone",
  storageGb: 64,
  wifi: true,
  bluetooth: true,
  gps: true,
  cameraConfiguration: "front+rear",
  microphone: true,
  accelerometer: true,
  gyroscope: true,
  compass: true,
  lightSensor: true,
  proximitySensor: true,
  batteryPercent: 100,
  charging: false,
  tabletResources: false
};

const builtInIds = new Set(["phone", "gaming-phone", "tablet", "large-tablet", "foldable"]);

export default function DeviceProfiles({ profiles, onChanged }: Props) {
  const [profile, setProfile] = useState<DeviceProfile>(blankProfile);
  const [busy, setBusy] = useState(false);
  const [output, setOutput] = useState("");

  const save = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      const saved = await invoke<DeviceProfile>("save_device_profile", { profile });
      setOutput(`Saved ${saved.name}.`);
      await onChanged();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (item: DeviceProfile) => {
    if (!window.confirm(`Remove custom profile ${item.name}?`)) return;
    setBusy(true);
    try {
      await invoke("remove_device_profile", { id: item.id });
      setOutput(`Removed ${item.name}.`);
      await onChanged();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Device Profiles</p><h3>Android form factors</h3></div>
        <span className="pill">{profiles.length} available</span>
      </div>

      <div className="image-manager-grid">
        <div className="profile-grid">
          {profiles.map(item => (
            <article className="instance-card" key={item.id}>
              <div className="instance-title"><strong>{item.name}</strong><span>{item.refreshRate} Hz</span></div>
              <p>{item.width}×{item.height} · {item.dpi} DPI</p>
              <small>{item.defaultCpuCores} vCPU · {Math.round(item.defaultRamMb / 1024)} GB RAM · {item.touchPoints} touch points</small>
              <small>{item.formFactor} · {item.storageGb} GB storage · Telephony {item.telephony ? "enabled" : "disabled"}</small>
              <small>Wi-Fi {item.wifi ? "on" : "off"} · Bluetooth {item.bluetooth ? "on" : "off"} · GPS {item.gps ? "on" : "off"} · Camera {item.cameraConfiguration}</small>
              {!builtInIds.has(item.id) && (
                <div className="button-row">
                  <button className="danger compact" disabled={busy} onClick={() => remove(item)}>Remove</button>
                </div>
              )}
            </article>
          ))}
        </div>

        <form className="image-register-form" onSubmit={save}>
          <h4>Create custom profile</h4>
          <label>Profile ID<input value={profile.id} onChange={e => setProfile({...profile,id:e.target.value})} /></label>
          <label>Name<input value={profile.name} onChange={e => setProfile({...profile,name:e.target.value})} /></label>

          <div className="split-fields">
            <label>Width<input type="number" min="320" max="8192" value={profile.width} onChange={e => setProfile({...profile,width:Number(e.target.value)})} /></label>
            <label>Height<input type="number" min="320" max="8192" value={profile.height} onChange={e => setProfile({...profile,height:Number(e.target.value)})} /></label>
          </div>

          <div className="split-fields">
            <label>DPI<input type="number" min="72" max="1000" value={profile.dpi} onChange={e => setProfile({...profile,dpi:Number(e.target.value)})} /></label>
            <label>Refresh Hz<input type="number" min="30" max="360" value={profile.refreshRate} onChange={e => setProfile({...profile,refreshRate:Number(e.target.value)})} /></label>
          </div>

          <div className="split-fields">
            <label>CPU cores<input type="number" min="1" max="64" value={profile.defaultCpuCores} onChange={e => setProfile({...profile,defaultCpuCores:Number(e.target.value)})} /></label>
            <label>RAM MB<input type="number" min="512" step="512" value={profile.defaultRamMb} onChange={e => setProfile({...profile,defaultRamMb:Number(e.target.value)})} /></label>
          </div>

          <div className="split-fields">
            <label>Touch points<input type="number" min="1" max="20" value={profile.touchPoints} onChange={e => setProfile({...profile,touchPoints:Number(e.target.value)})} /></label>
            <label>Form factor
              <select value={profile.formFactor} onChange={e => setProfile({...profile,formFactor:e.target.value})}>
                <option value="phone">Phone</option>
                <option value="tablet">Tablet</option>
                <option value="foldable">Foldable</option>
              </select>
            </label>
          </div>

          <label>Storage GB<input type="number" min="4" max="2048" value={profile.storageGb} onChange={e => setProfile({...profile,storageGb:Number(e.target.value)})} /></label>
          <label>Camera
            <select value={profile.cameraConfiguration} onChange={e => setProfile({...profile,cameraConfiguration:e.target.value as DeviceProfile["cameraConfiguration"]})}>
              <option value="none">None</option><option value="front">Front</option><option value="rear">Rear</option><option value="front+rear">Front + rear</option>
            </select>
          </label>
          <label>Battery %<input type="number" min="0" max="100" value={profile.batteryPercent} onChange={e => setProfile({...profile,batteryPercent:Number(e.target.value)})} /></label>
          {[
            ["telephony","Telephony capability"],["wifi","Wi-Fi capability"],["bluetooth","Bluetooth capability"],["gps","GPS capability"],
            ["microphone","Microphone"],["accelerometer","Accelerometer"],["gyroscope","Gyroscope"],["compass","Compass"],
            ["lightSensor","Light sensor"],["proximitySensor","Proximity sensor"],["charging","Charging state"],["tabletResources","Tablet resources"]
          ].map(([key,label]) => (
            <label className="checkbox-line" key={key}>
              <input type="checkbox" checked={Boolean(profile[key as keyof DeviceProfile])} onChange={e => setProfile({...profile,[key]:e.target.checked})} />
              {label}
            </label>
          ))}

          <button className="primary" disabled={busy}>{busy ? "Saving..." : "Save Custom Profile"}</button>
          {output && <pre className="inline-output">{output}</pre>}
        </form>
      </div>
    </section>
  );
}
