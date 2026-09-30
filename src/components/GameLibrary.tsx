import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AdbResult, AndroidInstance, GameSettings } from "../types";

export default function GameLibrary({ instances }: { instances: AndroidInstance[] }) {
  const [instanceId, setInstanceId] = useState(instances[0]?.id ?? "");
  const [packages, setPackages] = useState<string[]>([]);
  const [filter, setFilter] = useState("");
  const [status, setStatus] = useState("");
  const [busyPackage, setBusyPackage] = useState<string | null>(null);
  const [gameSettings, setGameSettings] = useState<GameSettings[]>([]);
  const [editing, setEditing] = useState<GameSettings | null>(null);

  const selected = useMemo(
    () => instances.find(instance => instance.id === instanceId) ?? instances[0],
    [instances, instanceId]
  );

  useEffect(() => {
    if (!instanceId && instances[0]) setInstanceId(instances[0].id);
  }, [instances, instanceId]);

  const refresh = async () => {
    if (!selected) return;
    try {
      const [result, saved] = await Promise.all([
        invoke<string[]>("adb_user_packages", { port: selected.adbPort }),
        invoke<GameSettings[]>("list_game_settings")
      ]);
      setPackages(result);
      setGameSettings(saved);
      setStatus(`Loaded ${result.length} user-installed packages.`);
    } catch (error) {
      setStatus(String(error));
    }
  };

  useEffect(() => {
    if (selected) refresh().catch(() => undefined);
  }, [selected?.id]);

  const run = async (packageName: string) => {
    if (!selected) return;
    setBusyPackage(packageName);
    try {
      const results = await invoke<AdbResult[]>("launch_game_with_settings", {
        port: selected.adbPort,
        packageName
      });
      const result = results[results.length - 1];
      setStatus(result?.success ? `Launched ${packageName} with saved game settings.` : result?.stderr || result?.stdout || "Launch failed");
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusyPackage(null);
    }
  };

  const editSettings = (packageName: string) => {
    const saved = gameSettings.find(item => item.packageName === packageName);
    setEditing(saved ?? {
      packageName,
      renderer: "auto",
      androidVersion: "16",
      orientation: "automatic",
      dpi: 420,
      fps: 60,
      keymapId: null,
      aiSkillId: null,
      compatibilityRating: "unknown",
      knownIssues: [],
      crashDiagnostics: true,
      notes: ""
    });
  };

  const saveSettings = async () => {
    if (!editing) return;
    try {
      const saved = await invoke<GameSettings>("save_game_settings", { settings: editing });
      setGameSettings(current => [...current.filter(item => item.packageName !== saved.packageName), saved]);
      setEditing(saved);
      setStatus(`Saved settings for ${saved.packageName}.`);
    } catch (error) {
      setStatus(String(error));
    }
  };

  const uninstall = async (packageName: string) => {
    if (!selected || !window.confirm(`Uninstall ${packageName}?`)) return;
    setBusyPackage(packageName);
    try {
      const result = await invoke<AdbResult>("adb_uninstall", {
        port: selected.adbPort,
        packageName
      });
      setStatus(result.success ? `Uninstalled ${packageName}.` : result.stderr || result.stdout);
      if (result.success) await refresh();
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusyPackage(null);
    }
  };

  const visible = packages.filter(pkg => pkg.toLowerCase().includes(filter.toLowerCase()));

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Game Library</p><h3>User-installed Android apps and games</h3></div>
        <span className="pill">{packages.length} installed</span>
      </div>

      <div className="tool-column">
        <label>Instance
          <select value={selected?.id ?? ""} onChange={e => setInstanceId(e.target.value)}>
            {instances.map(instance => <option key={instance.id} value={instance.id}>{instance.name}</option>)}
          </select>
        </label>

        <div className="split-fields">
          <label>Filter packages<input value={filter} onChange={e => setFilter(e.target.value)} /></label>
          <div className="button-row"><button className="ghost compact" onClick={refresh}>Refresh</button></div>
        </div>

        <div className="profile-grid">
          {visible.map(packageName => (
            <article className="instance-card" key={packageName}>
              <div className="instance-title"><strong>{packageName}</strong></div>
              <small>User-installed package</small>
              <div className="button-row">
                <button className="primary compact" disabled={busyPackage === packageName} onClick={() => run(packageName)}>Launch</button>
                <button className="ghost compact" disabled={busyPackage === packageName} onClick={() => editSettings(packageName)}>Settings</button>
                <button className="danger compact" disabled={busyPackage === packageName} onClick={() => uninstall(packageName)}>Uninstall</button>
              </div>
            </article>
          ))}
        </div>

        {editing && (
          <div className="tool-group">
            <h4>Per-game settings · {editing.packageName}</h4>
            <div className="split-fields">
              <label>Renderer<select value={editing.renderer ?? "auto"} onChange={e => setEditing({...editing,renderer:e.target.value as GameSettings["renderer"]})}>
                {["auto","vulkan","opengl","directx","software"].map(v => <option key={v} value={v}>{v}</option>)}
              </select></label>
              <label>Android<select value={editing.androidVersion ?? "16"} onChange={e => setEditing({...editing,androidVersion:e.target.value as GameSettings["androidVersion"]})}>
                {["9","10","11","12","12L","13","14","15","16"].map(v => <option key={v} value={v}>Android {v}</option>)}
              </select></label>
            </div>
            <div className="split-fields">
              <label>Orientation<select value={editing.orientation ?? "automatic"} onChange={e => setEditing({...editing,orientation:e.target.value as GameSettings["orientation"]})}>
                {["automatic","portrait","landscape","reverse-portrait","reverse-landscape"].map(v => <option key={v} value={v}>{v}</option>)}
              </select></label>
              <label>DPI<input type="number" min="72" max="1000" value={editing.dpi ?? 420} onChange={e => setEditing({...editing,dpi:Number(e.target.value)})}/></label>
              <label>FPS<select value={editing.fps ?? 60} onChange={e => setEditing({...editing,fps:Number(e.target.value)})}>
                {[30,60,90,120,144,165,240].map(v => <option key={v} value={v}>{v}</option>)}
              </select></label>
            </div>
            <div className="split-fields">
              <label>Keymap ID<input value={editing.keymapId ?? ""} onChange={e => setEditing({...editing,keymapId:e.target.value || null})}/></label>
              <label>AI skill ID<input value={editing.aiSkillId ?? ""} onChange={e => setEditing({...editing,aiSkillId:e.target.value || null})}/></label>
            </div>
            <label>Compatibility<select value={editing.compatibilityRating ?? "unknown"} onChange={e => setEditing({...editing,compatibilityRating:e.target.value as GameSettings["compatibilityRating"]})}>
              {["unknown","good","partial","broken"].map(v => <option key={v} value={v}>{v}</option>)}
            </select></label>
            <label>Known issues<textarea value={editing.knownIssues.join("\n")} onChange={e => setEditing({...editing,knownIssues:e.target.value.split(/\r?\n/).map(v=>v.trim()).filter(Boolean)})}/></label>
            <label className="checkbox-line"><input type="checkbox" checked={editing.crashDiagnostics} onChange={e => setEditing({...editing,crashDiagnostics:e.target.checked})}/>Collect crash diagnostics for this game</label>
            <label>Notes<textarea value={editing.notes} onChange={e => setEditing({...editing,notes:e.target.value})}/></label>
            <div className="button-row"><button className="primary compact" onClick={saveSettings}>Save game settings</button><button className="ghost compact" onClick={()=>setEditing(null)}>Close</button></div>
          </div>
        )}

        {status && <pre className="inline-output">{status}</pre>}
      </div>
    </section>
  );
}
