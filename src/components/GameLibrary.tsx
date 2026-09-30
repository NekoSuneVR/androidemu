import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AdbResult, AndroidInstance } from "../types";

export default function GameLibrary({ instances }: { instances: AndroidInstance[] }) {
  const [instanceId, setInstanceId] = useState(instances[0]?.id ?? "");
  const [packages, setPackages] = useState<string[]>([]);
  const [filter, setFilter] = useState("");
  const [status, setStatus] = useState("");
  const [busyPackage, setBusyPackage] = useState<string | null>(null);

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
      const result = await invoke<string[]>("adb_user_packages", { port: selected.adbPort });
      setPackages(result);
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
      const result = await invoke<AdbResult>("adb_launch_package", {
        port: selected.adbPort,
        packageName
      });
      setStatus(result.success ? `Launched ${packageName}.` : result.stderr || result.stdout);
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusyPackage(null);
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
                <button className="danger compact" disabled={busyPackage === packageName} onClick={() => uninstall(packageName)}>Uninstall</button>
              </div>
            </article>
          ))}
        </div>

        {status && <pre className="inline-output">{status}</pre>}
      </div>
    </section>
  );
}
