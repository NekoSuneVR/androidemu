import { FormEvent, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { AdbInfo, AdbResult, AndroidInstance, RuntimeLogs } from "../types";

type Props = {
  instances: AndroidInstance[];
  adbInfo: AdbInfo | null;
};

export default function DeveloperTools({ instances, adbInfo }: Props) {
  const [selectedId, setSelectedId] = useState(instances[0]?.id ?? "");
  const [command, setCommand] = useState("getprop ro.build.version.release");
  const [rootCommand, setRootCommand] = useState("id");
  const [apkPath, setApkPath] = useState("");
  const [apkBatch, setApkBatch] = useState("");
  const [packageName, setPackageName] = useState("");
  const [screenshotPath, setScreenshotPath] = useState("nekodroid-screenshot.png");
  const [recordingPath, setRecordingPath] = useState("nekodroid-recording.mp4");
  const [recordingSeconds, setRecordingSeconds] = useState(15);
  const [pushSource, setPushSource] = useState("");
  const [pushSources, setPushSources] = useState("");
  const [pushDestination, setPushDestination] = useState("/sdcard/Download/");
  const [pullSource, setPullSource] = useState("/sdcard/Download/");
  const [pullDestination, setPullDestination] = useState("");
  const [forwardLocal, setForwardLocal] = useState("tcp:8080");
  const [forwardRemote, setForwardRemote] = useState("tcp:8080");
  const [reverseRemote, setReverseRemote] = useState("tcp:3000");
  const [reverseLocal, setReverseLocal] = useState("tcp:3000");
  const [logcatLines, setLogcatLines] = useState(300);
  const [touchX, setTouchX] = useState(540);
  const [touchY, setTouchY] = useState(1200);
  const [dragX, setDragX] = useState(900);
  const [dragY, setDragY] = useState(1200);
  const [touchDuration, setTouchDuration] = useState(600);
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

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!selected || busy) return;

      if (event.key === "F9" && screenshotPath) {
        event.preventDefault();
        run("adb_screenshot", { destination: screenshotPath });
        return;
      }

      if (event.ctrlKey && event.altKey && (event.key === "ArrowLeft" || event.key === "ArrowRight")) {
        event.preventDefault();
        run("adb_rotate_orientation", {
          direction: event.key === "ArrowLeft" ? "left" : "right"
        });
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  });
  const selectApk = async () => {
    try {
      const selectedPath = await open({
        multiple: false,
        directory: false,
        title: "Select APK",
        filters: [{ name: "Android Package", extensions: ["apk"] }]
      });
      if (selectedPath) setApkPath(selectedPath);
    } catch (error) {
      setOutput(String(error));
    }
  };

  const shell = (event: FormEvent) => {
    event.preventDefault();
    run("adb_shell", { command });
  };

  const readRuntimeLogs = async () => {
    if (!selected) return;
    setBusy(true);
    try {
      const logs = await invoke<RuntimeLogs>("get_instance_logs", { id: selected.id });
      setOutput([
        "=== QEMU STDOUT ===",
        logs.stdout || "(empty)",
        "",
        "=== QEMU STDERR ===",
        logs.stderr || "(empty)",
        "",
        "=== CRASH REPORT ===",
        logs.crashReport || "(none)"
      ].join("\n"));
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
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
                <h4>Root tools</h4>
                <div className="button-row">
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_root", {})}>ADB Root</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_unroot", {})}>ADB Unroot</button>
                </div>
                <label>Root shell command<input value={rootCommand} onChange={e => setRootCommand(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy || !rootCommand} onClick={() => run("adb_root_shell", { command: rootCommand })}>Run with su</button>
                <div className="warning-box">
                  Root only works when the selected Android image supports root or su. Root can break app compatibility and may cause Play Integrity checks to fail.
                </div>
              </div>

              <div className="tool-group">
                <label>APK path<input placeholder="C:\\Downloads\\game.apk" value={apkPath} onChange={e => setApkPath(e.target.value)} /></label>
                <div className="button-row">
                  <button className="ghost compact" disabled={busy} onClick={selectApk}>Select APK from PC</button>
                  <button className="ghost compact" disabled={busy || !apkPath} onClick={() => run("adb_install", { apkPath })}>Install APK</button>
                </div>
              </div>

              <div className="tool-group">
                <h4>Batch / split APK install</h4>
                <label>APK paths, one per line<textarea placeholder={"C:\\Downloads\\base.apk\nC:\\Downloads\\config.en.apk"} value={apkBatch} onChange={e => setApkBatch(e.target.value)} /></label>
                <div className="button-row">
                  <button
                    className="ghost compact"
                    disabled={busy || !apkBatch.trim()}
                    onClick={() => run("adb_install_batch", { apkPaths: apkBatch.split(/\r?\n/).map(v => v.trim()).filter(Boolean) })}
                  >
                    Install each APK
                  </button>
                  <button
                    className="ghost compact"
                    disabled={busy || apkBatch.split(/\r?\n/).filter(v => v.trim()).length < 2}
                    onClick={() => run("adb_install_multiple", { apkPaths: apkBatch.split(/\r?\n/).map(v => v.trim()).filter(Boolean) })}
                  >
                    Install split APK set
                  </button>
                </div>
              </div>

              <div className="tool-group">
                <label>Package name<input placeholder="com.example.game" value={packageName} onChange={e => setPackageName(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy || !packageName} onClick={() => run("adb_uninstall", { packageName })}>Uninstall package</button>
              </div>

              <div className="button-row">
                <button className="ghost compact" disabled={busy} onClick={readRuntimeLogs}>Runtime logs</button>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_device_info", {})}>Device info</button>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_reboot", { mode: null })}>Reboot Android</button>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_reboot", { mode: "recovery" })}>Recovery</button>
                <button className="ghost compact" disabled={busy} onClick={() => run("adb_reboot", { mode: "bootloader" })}>Bootloader</button>
              </div>

              <div className="tool-group">
                <label>Screenshot host path<input value={screenshotPath} onChange={e => setScreenshotPath(e.target.value)} /></label>
                <div className="button-row">
                  <button className="ghost compact" disabled={busy || !screenshotPath} onClick={() => run("adb_screenshot", { destination: screenshotPath })}>Save screenshot</button>
                  <span className="pill">F9 hotkey</span>
                </div>
              </div>

              <div className="tool-group">
                <h4>Screen recording</h4>
                <label>Recording host path<input value={recordingPath} onChange={e => setRecordingPath(e.target.value)} /></label>
                <label>Seconds<input type="number" min="1" max="180" value={recordingSeconds} onChange={e => setRecordingSeconds(Number(e.target.value))} /></label>
                <button className="ghost compact" disabled={busy || !recordingPath} onClick={() => run("adb_screen_record", { destination: recordingPath, seconds: recordingSeconds })}>Record Android screen</button>
              </div>

              <div className="tool-group">
                <label>Host file or folder<input value={pushSource} onChange={e => setPushSource(e.target.value)} /></label>
                <label>Android destination<input value={pushDestination} onChange={e => setPushDestination(e.target.value)} /></label>
                <button className="ghost compact" disabled={busy || !pushSource} onClick={() => run("adb_push", { source: pushSource, destination: pushDestination })}>Push file</button>

                <label>Multiple host files/folders, one per line
                  <textarea value={pushSources} onChange={e => setPushSources(e.target.value)} />
                </label>
                <button
                  className="ghost compact"
                  disabled={busy || !pushSources.trim()}
                  onClick={() => run("adb_push_multiple", {
                    sources: pushSources.split(/\r?\n/).map(v => v.trim()).filter(Boolean),
                    destination: pushDestination
                  })}
                >
                  Push multiple
                </button>
              </div>

              <div className="tool-group">
                <label>Android file or folder<input value={pullSource} onChange={e => setPullSource(e.target.value)} /></label>
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
                <h4>Orientation</h4>
                <div className="button-row">
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_set_orientation", { orientation: "auto" })}>Auto</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_set_orientation", { orientation: "portrait" })}>Portrait</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_set_orientation", { orientation: "landscape" })}>Landscape</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_set_orientation", { orientation: "reverse-portrait" })}>Reverse portrait</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_set_orientation", { orientation: "reverse-landscape" })}>Reverse landscape</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_rotate_orientation", { direction: "left" })}>Rotate left</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_rotate_orientation", { direction: "right" })}>Rotate right</button>
                </div>
                <small className="muted">Hotkeys: Ctrl+Alt+Left / Ctrl+Alt+Right</small>
              </div>

              <div className="tool-group">
                <h4>Virtual touch test</h4>
                <div className="split-fields">
                  <label>X<input type="number" min="0" max="32767" value={touchX} onChange={e => setTouchX(Number(e.target.value))} /></label>
                  <label>Y<input type="number" min="0" max="32767" value={touchY} onChange={e => setTouchY(Number(e.target.value))} /></label>
                </div>
                <div className="split-fields">
                  <label>Drag X<input type="number" min="0" max="32767" value={dragX} onChange={e => setDragX(Number(e.target.value))} /></label>
                  <label>Drag Y<input type="number" min="0" max="32767" value={dragY} onChange={e => setDragY(Number(e.target.value))} /></label>
                </div>
                <label>Duration ms<input type="number" min="50" max="5000" value={touchDuration} onChange={e => setTouchDuration(Number(e.target.value))} /></label>
                <div className="button-row">
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_input_tap", { x: touchX, y: touchY })}>Tap</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_input_double_tap", { x: touchX, y: touchY })}>Double tap</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_input_hold", { x: touchX, y: touchY, durationMs: touchDuration })}>Hold</button>
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_input_swipe", { x1: touchX, y1: touchY, x2: dragX, y2: dragY, durationMs: touchDuration })}>Drag / swipe</button>
                </div>
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
                  <button className="ghost compact" disabled={busy} onClick={() => run("adb_surfaceflinger_info", {})}>SurfaceFlinger</button>
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
