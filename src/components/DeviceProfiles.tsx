import type { DeviceProfile } from "../types";

export default function DeviceProfiles({ profiles }: { profiles: DeviceProfile[] }) {
  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Device Profiles</p><h3>Android form factors</h3></div>
        <span className="pill">{profiles.length} built in</span>
      </div>
      <div className="profile-grid">
        {profiles.map(profile => (
          <article className="instance-card" key={profile.id}>
            <div className="instance-title"><strong>{profile.name}</strong><span>{profile.refreshRate} Hz</span></div>
            <p>{profile.width}×{profile.height} · {profile.dpi} DPI</p>
            <small>{profile.defaultCpuCores} vCPU · {Math.round(profile.defaultRamMb / 1024)} GB RAM · {profile.touchPoints} touch points</small>
            <small>{profile.formFactor} · Telephony {profile.telephony ? "enabled" : "disabled"}</small>
          </article>
        ))}
      </div>
    </section>
  );
}
