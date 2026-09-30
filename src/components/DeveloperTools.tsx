import { FormEvent, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AdbInfo, AdbResult, AndroidInstance } from "../types";

type Props = {
  instances: AndroidInstance[];
  adbInfo: AdbInfo | null;
};

export default function DeveloperTools({ instances, adbInfo }: Props) {
  const [selectedId, setSelectedId] = useState(instances[0]?.id ?? "");
  const [command, setCommand] = useState("getprop ro.build.version.release");
  const [apkPath, setApkPath] = useState("");
  const [packageName, setPackageName] = useState("");
  const [screenshotPath, setScreenshotPath] = useState("nekodroid-screenshot.png");
  const [pushSource, setPushSource] = useState("");
  const [pushDestination, setPushDestination] = useState("/sdcard/Download/");
  const [pullSource, setPullSource] = useState("/sdcard/Download/");
  const [pullDestination, setPullDestination] = useState("");
  const [forwardLocal, setForwardLocal] = useState("tcp:8080");
  const [forwardRemote, setForwardRemote] = useState("tcp:8080");
  const [reverseRemote, setReverseRemote] = useState("tcp:3000");
  const [reverseLocal, setReverseLocal] = useState("tcp:3000");
  const [logcatLines, setLogcatLines] = useState(300);
  const [output, setOutput] = useState("ADB output will appear here.");
  const [busy, setBusy] = useState(false);

  const selected = useMemo(
    () => instances.find(instance => instance.id === selectedId) ?? instances[0],
    [instances, selectedId]
  );

  const run = async (name: string, args: Record<string, unknown>) => {
    if (!selected) return;
    setBusy(true);
    try {
      const result = await invoke<AdbResult>(name, { port: selected.adbPort, ...args });
      setOutput([
        result.success ? "SUCCESS" : `FAILED (exit ${result.exitCode ?? "unknown"})`,
        result.stdout,
        result.stderr
      ].filter(Boolean).join("\n\n"));
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const shell = (event: FormEvent) => {
    event.preventDefault();
    run("adb_shell", { command });
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Developer Tools</p><h3>ADB console and file bridge</h3></div>
        <span className={adbInfo?.found ? "pill" : "pill pill-warn"}>{adbInfo?.found ? "ADB detected" : "ADB missing"}</span>
      </div>

      <div className="developer-grid">
        <div className="tool-column">
          <label>Instance
            <select value={selected?.id ?? ""} onChange={e => setSelectedId(e.target.value)}>
              {instances.map(instance => <option key={instance.id} value={instance.id}>{instance.name} · localhost:{instance.adbPort}</option>)}
            </select>
          </label>
          {selected ? (
            <>
              <div className="button-row">
                <button className="primary compact" disabled={busy} onClick={() => run("adb_connect", {})}>Connect</button>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_get_state", {})}>State</button>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_disconnect", {})}>Disconnect</button>
              </div>

              <form onSubmit={shell}>
                <label>ADB shell command<input value={command} onChange={e => setCommand(e.target.value)} /></label>
                <button className="primary compact" disabled={busy}>Run shell</button>
              </form>

              <div className="tool-group">
                <label>APK path<input placeholder="C:\\Downloads\\game.apk" value={apkPath} onChange={e => setApkPath(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy || !apkPath} onClick={() => run("adb_install", { apkPath })}>Install APK</button>
              </div>

              <div className="tool-group">
                <label>Package name<input placeholder="com.example.game" value={packageName} onChange={e => setPackageName(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy || !packageName} onClick={() => run("adb_uninstall", { packageName })}>Uninstall package</button>
              </div>

              <div className="button-row">
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_device_info", {})}>Device info</button>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_reboot", { mode: null })}>Reboot Android</button>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_reboot", { mode: "recovery" })}>Recovery</button>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_reboot", { mode: "bootloader" })}>Bootloader</button>
              </div>

              <div className="tool-group">
                <label>Screenshot host path<input value={screenshotPath} onChange={e => setScreenshotPath(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy || !screenshotPath} onClick={() => run("adb_screenshot", { destination: screenshotPath })}>Save screenshot</button>
              </div>

              <div className="tool-group">
                <label>Host file<input value={pushSource} onChange={e => setPushSource(e.target.value)} /></label>
                <label>Android destination<input value={pushDestination} onChange={e => setPushDestination(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy || !pushSource} onClick={() => run("adb_push", { source: pushSource, destination: pushDestination })}>Push file</button>
              </div>

              <div className="tool-group">
                <label>Android source<input value={pullSource} onChange={e => setPullSource(e.target.value)} /></label>
                <label>Host destination<input value={pullDestination} onChange={e => setPullDestination(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy || !pullDestination} onClick={() => run("adb_pull", { source: pullSource, destination: pullDestination })}>Pull file</button>
              </div>

              <div className="tool-group">
                <h4>Port forwarding</h4>
                <label>Host/local socket<input value={forwardLocal} onChange={e => setForwardLocal(e.target.value)} /></label>
                <label>Android/remote socket<input value={forwardRemote} onChange={e => setForwardRemote(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_forward", { local: forwardLocal, remote: forwardRemote })}>ADB forward</button>
              </div>

              <div className="tool-group">
                <h4>Reverse forwarding</h4>
                <label>Android/remote socket<input value={reverseRemote} onChange={e => setReverseRemote(e.target.value)} /></label>
                <label>Host/local socket<input value={reverseLocal} onChange={e => setReverseLocal(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_reverse", { remote: reverseRemote, local: reverseLocal })}>ADB reverse</button>
              </div>

              <div className="tool-group">
                <h4>Logcat</h4>
                <label>Lines<input type="number" min="1" max="5000" value={logcatLines} onChange={e => setLogcatLines(Number(e.target.value))} /></label>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_logcat", { lines: logcatLines })}>Read Logcat</button>
              </div>

              <div className="tool-group">
                <h4>Inspect Android</h4>
                <div className="button-row">
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_packages", {})}>Packages</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_processes", {})}>Processes</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_properties", {})}>Properties</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_build_properties", {})}>Build props</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_storage_info", {})}>Storage</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_activities", {})}>Activities</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_services", {})}>Services</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_network_connections", {})}>Network</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_kernel_log", { lines: logcatLines })}>Kernel log</button>
                </div>
              </div>
            </>
          ) : <p className="muted">Create an Android instance before using ADB tools.</p>}
        </div>

        <div className="terminal">
          <div className="terminal-title">ADB output</div>
          <pre>{output}</pre>
        </div>
      </div>
    </section>
  );
}
