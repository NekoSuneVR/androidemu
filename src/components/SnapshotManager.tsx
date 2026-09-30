import { FormEvent, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AndroidInstance, SnapshotInfo } from "../types";

export default function SnapshotManager({ instances }: { instances: AndroidInstance[] }) {
  const [instanceId, setInstanceId] = useState(instances[0]?.id ?? "");
  const [snapshots, setSnapshots] = useState<SnapshotInfo[]>([]);
  const [name, setName] = useState("Manual snapshot");
  const [description, setDescription] = useState("");
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);

  const selected = useMemo(
    () => instances.find(instance => instance.id === instanceId) ?? instances[0],
    [instances, instanceId]
  );

  const refresh = async () => {
    if (!selected) {
      setSnapshots([]);
      return;
    }
    const result = await invoke<SnapshotInfo[]>("list_snapshots", { instanceId: selected.id });
    setSnapshots(result);
  };

  useEffect(() => {
    if (!instanceId && instances[0]) setInstanceId(instances[0].id);
  }, [instances, instanceId]);

  useEffect(() => {
    refresh().catch(error => setOutput(String(error)));
  }, [selected?.id]);

  const create = async (event: FormEvent) => {
    event.preventDefault();
    if (!selected) return;
    setBusy(true);
    try {
      const created = await invoke<SnapshotInfo>("create_snapshot", {
        instanceId: selected.id,
        name,
        description
      });
      setOutput(`Created snapshot ${created.name}.`);
      await refresh();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const rename = async (snapshot: SnapshotInfo) => {
    if (!selected) return;
    const nextName = window.prompt("Snapshot name", snapshot.name)?.trim();
    if (!nextName) return;
    const nextDescription = window.prompt("Snapshot description", snapshot.description) ?? snapshot.description;
    setBusy(true);
    try {
      await invoke<SnapshotInfo>("rename_snapshot", {
        instanceId: selected.id,
        snapshotId: snapshot.id,
        name: nextName,
        description: nextDescription
      });
      setOutput(`Renamed snapshot to ${nextName}.`);
      await refresh();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const restore = async (snapshot: SnapshotInfo) => {
    if (!selected || !window.confirm(`Restore ${snapshot.name}? Current unsaved VM disk changes after this snapshot will be replaced.`)) return;
    setBusy(true);
    try {
      await invoke("restore_snapshot", { instanceId: selected.id, snapshotId: snapshot.id });
      setOutput(`Restored ${snapshot.name}.`);
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (snapshot: SnapshotInfo) => {
    if (!selected || !window.confirm(`Delete snapshot ${snapshot.name}?`)) return;
    setBusy(true);
    try {
      await invoke("delete_snapshot", { instanceId: selected.id, snapshotId: snapshot.id });
      setOutput(`Deleted ${snapshot.name}.`);
      await refresh();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Snapshots</p><h3>QCOW2 instance snapshots</h3></div>
        <span className="pill">{snapshots.length} saved</span>
      </div>

      <div className="image-manager-grid">
        <div>
          <label>Instance
            <select value={selected?.id ?? ""} onChange={e => setInstanceId(e.target.value)}>
              {instances.map(instance => (
                <option key={instance.id} value={instance.id}>
                  {instance.name} · {instance.status}
                </option>
              ))}
            </select>
          </label>

          <div className="image-list">
            {snapshots.length === 0 ? (
              <p className="muted">No snapshots for this instance yet.</p>
            ) : snapshots.map(snapshot => (
              <article className="instance-card" key={snapshot.id}>
                <div className="instance-title">
                  <strong>{snapshot.name}</strong>
                  <span>{new Date(snapshot.createdAt * 1000).toLocaleString()}</span>
                </div>
                {snapshot.description && <p>{snapshot.description}</p>}
                <small>{snapshot.id}</small>
                <div className="button-row">
                  <button className="ghost compact" disabled={busy || selected?.status === "running"} onClick={() => rename(snapshot)}>Rename</button>
                  <button className="primary compact" disabled={busy || selected?.status === "running"} onClick={() => restore(snapshot)}>Restore</button>
                  <button className="danger compact" disabled={busy || selected?.status === "running"} onClick={() => remove(snapshot)}>Delete</button>
                </div>
              </article>
            ))}
          </div>
        </div>

        <form className="image-register-form" onSubmit={create}>
          <h4>Create snapshot</h4>
          <label>Name<input value={name} onChange={e => setName(e.target.value)} /></label>
          <label>Description<textarea value={description} onChange={e => setDescription(e.target.value)} /></label>
          <button className="primary" disabled={busy || !selected || selected.status === "running"}>
            {busy ? "Working..." : "Create Snapshot"}
          </button>
          <div className="button-row">
            <button type="button" className="ghost compact" disabled={busy || !selected || selected.status === "running"} onClick={async()=>{if(!selected)return;try{setBusy(true);await invoke("create_clean_snapshot",{instanceId:selected.id});await refresh();setOutput("Created clean snapshot.");}catch(error){setOutput(String(error));}finally{setBusy(false);}}}>Clean snapshot</button>
            <button type="button" className="ghost compact" disabled={busy || !selected || selected.status === "running"} onClick={async()=>{if(!selected)return;try{setBusy(true);await invoke("create_rooted_snapshot",{instanceId:selected.id});await refresh();setOutput("Created rooted snapshot.");}catch(error){setOutput(String(error));}finally{setBusy(false);}}}>Rooted snapshot</button>
            <button type="button" className="danger compact" disabled={busy || !selected || selected.status === "running"} onClick={async()=>{if(!selected||!confirm("Keep the newest 10 snapshots and delete older ones?"))return;try{setBusy(true);const removed=await invoke<number>("cleanup_snapshots",{instanceId:selected.id,keep:10});await refresh();setOutput(`Removed ${removed} old snapshot(s).`);}catch(error){setOutput(String(error));}finally{setBusy(false);}}}>Cleanup old snapshots</button>
          </div>
          <small className="muted">Snapshots require the instance to be stopped and its runtime QCOW2 disk to exist.</small>
          {output && <pre className="inline-output">{output}</pre>}
        </form>
      </div>
    </section>
  );
}
