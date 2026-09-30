import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AdbResult, AndroidFileEntry, AndroidInstance } from "../types";

export default function FileManager({ instances }: { instances: AndroidInstance[] }) {
  const [instanceId, setInstanceId] = useState(instances[0]?.id ?? "");
  const [path, setPath] = useState("/sdcard");
  const [entries, setEntries] = useState<AndroidFileEntry[]>([]);
  const [search, setSearch] = useState("");
  const [hostUpload, setHostUpload] = useState("");
  const [androidDownload, setAndroidDownload] = useState("/sdcard/Download/");
  const [hostDownload, setHostDownload] = useState("");
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);

  const selected = useMemo(
    () => instances.find(instance => instance.id === instanceId) ?? instances[0],
    [instances, instanceId]
  );

  useEffect(() => {
    if (!instanceId && instances[0]) setInstanceId(instances[0].id);
  }, [instances, instanceId]);

  const run = async (name: string, args: Record<string, unknown>) => {
    if (!selected) return null;
    const result = await invoke<AdbResult>(name, { port: selected.adbPort, ...args });
    setOutput([
      result.success ? "SUCCESS" : `FAILED (exit ${result.exitCode ?? "unknown"})`,
      result.stdout,
      result.stderr
    ].filter(Boolean).join("\n\n"));
    return result;
  };

  const refresh = async (nextPath = path) => {
    if (!selected) return;
    setBusy(true);
    try {
      const result = await invoke<AndroidFileEntry[]>("adb_list_files", { port: selected.adbPort, path: nextPath });
      setPath(nextPath);
      setEntries(result);
      setOutput(`Loaded ${result.length} entries from ${nextPath}`);
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  useEffect(() => {
    if (selected) refresh(path).catch(() => undefined);
  }, [selected?.id]);

  const parentPath = () => {
    if (path === "/") return "/";
    const parts = path.split("/").filter(Boolean);
    parts.pop();
    return "/" + parts.join("/");
  };

  const createFolder = async () => {
    const name = window.prompt("Folder name")?.trim();
    if (!name || name.includes("/")) return;
    setBusy(true);
    try {
      await run("adb_make_directory", { path: path === "/" ? `/${name}` : `${path}/${name}` });
      await refresh();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (entry: AndroidFileEntry) => {
    if (!window.confirm(`Delete ${entry.path}?`)) return;
    setBusy(true);
    try {
      await run("adb_remove_path", { path: entry.path });
      await refresh();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const move = async (entry: AndroidFileEntry) => {
    const destination = window.prompt("Move/rename destination", entry.path)?.trim();
    if (!destination || destination === entry.path) return;
    setBusy(true);
    try {
      await run("adb_move_path", { source: entry.path, destination });
      await refresh();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const copy = async (entry: AndroidFileEntry) => {
    const destination = window.prompt("Copy destination", `${entry.path}.copy`)?.trim();
    if (!destination) return;
    setBusy(true);
    try {
      await run("adb_copy_path", { source: entry.path, destination });
      await refresh();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const properties = async (entry: AndroidFileEntry) => {
    setBusy(true);
    try {
      await run("adb_file_properties", { path: entry.path });
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const doSearch = async () => {
    if (!search.trim()) return;
    setBusy(true);
    try {
      await run("adb_search_files", { path, query: search });
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const upload = async () => {
    if (!hostUpload) return;
    setBusy(true);
    try {
      await run("adb_push", { source: hostUpload, destination: path });
      await refresh();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const download = async () => {
    if (!androidDownload || !hostDownload) return;
    setBusy(true);
    try {
      await run("adb_pull", { source: androidDownload, destination: hostDownload });
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Android File Manager</p><h3>Browse and transfer guest files</h3></div>
        <span className="pill">{entries.length} entries</span>
      </div>

      <div className="developer-grid">
        <div className="tool-column">
          <label>Instance
            <select value={selected?.id ?? ""} onChange={e => setInstanceId(e.target.value)}>
              {instances.map(instance => <option key={instance.id} value={instance.id}>{instance.name}</option>)}
            </select>
          </label>

          <label>Android path
            <input value={path} onChange={e => setPath(e.target.value)} onKeyDown={e => {
              if (e.key === "Enter") refresh(e.currentTarget.value);
            }} />
          </label>

          <div className="button-row">
            <button className="ghost compact" disabled={busy} onClick={() => refresh(parentPath())}>Up</button>
            <button className="ghost compact" disabled={busy} onClick={() => refresh("/sdcard")}>/sdcard</button>
            <button className="ghost compact" disabled={busy} onClick={() => refresh("/storage")}>/storage</button>
            <button className="ghost compact" disabled={busy} onClick={() => refresh()}>Refresh</button>
            <button className="ghost compact" disabled={busy} onClick={createFolder}>New Folder</button>
            <button className="ghost compact" disabled={busy} onClick={() => run("adb_storage_info", {})}>Storage Usage</button>
          </div>

          <div className="image-list">
            {entries.map(entry => (
              <article className="instance-card" key={entry.path}>
                <div className="instance-title">
                  <strong>{entry.isDir ? "📁" : "📄"} {entry.name}</strong>
                  <span>{entry.isDir ? "folder" : `${Math.round(entry.size / 1024)} KB`}</span>
                </div>
                <small>{entry.permissions || "permissions unknown"}</small>
                <div className="button-row">
                  {entry.isDir && <button className="primary compact" disabled={busy} onClick={() => refresh(entry.path)}>Open</button>}
                  <button className="ghost compact" disabled={busy} onClick={() => properties(entry)}>Properties</button>
                  <button className="ghost compact" disabled={busy} onClick={() => move(entry)}>Move / Rename</button>
                  <button className="ghost compact" disabled={busy} onClick={() => copy(entry)}>Copy</button>
                  <button className="danger compact" disabled={busy} onClick={() => remove(entry)}>Delete</button>
                </div>
              </article>
            ))}
          </div>

          <div className="tool-group">
            <h4>Search</h4>
            <label>Find name under current path<input value={search} onChange={e => setSearch(e.target.value)} /></label>
            <button className="ghost compact" disabled={busy || !search.trim()} onClick={doSearch}>Search</button>
          </div>

          <div className="tool-group">
            <h4>Upload</h4>
            <label>Host file or folder<input value={hostUpload} onChange={e => setHostUpload(e.target.value)} /></label>
            <button className="ghost compact" disabled={busy || !hostUpload} onClick={upload}>Upload to current folder</button>
          </div>

          <div className="tool-group">
            <h4>Download</h4>
            <label>Android file or folder<input value={androidDownload} onChange={e => setAndroidDownload(e.target.value)} /></label>
            <label>Host destination<input value={hostDownload} onChange={e => setHostDownload(e.target.value)} /></label>
            <button className="ghost compact" disabled={busy || !androidDownload || !hostDownload} onClick={download}>Download</button>
          </div>
        </div>

        <div className="terminal">
          <div className="terminal-title">File manager output</div>
          <pre>{output || "Connect ADB and choose a path to begin."}</pre>
        </div>
      </div>
    </section>
  );
}
