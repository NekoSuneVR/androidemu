import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AndroidInstance, AppSettings, SystemReadiness } from "../types";

export default function FirstRunWizard() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [readiness, setReadiness] = useState<SystemReadiness | null>(null);
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    Promise.all([
      invoke<AppSettings>("get_app_settings"),
      invoke<SystemReadiness>("get_system_readiness")
    ])
      .then(([appSettings, systemReadiness]) => {
        setSettings(appSettings);
        setReadiness(systemReadiness);
      })
      .catch(error => setStatus(String(error)));
  }, []);

  if (!settings || settings.firstRunCompleted) return null;

  const finish = async () => {
    if (!readiness) return;
    setBusy(true);
    try {
      const existing = await invoke<AndroidInstance[]>("list_instances");
      if (existing.length === 0) {
        await invoke<AndroidInstance>("create_instance", {
          request: {
            name: "Gaming",
            androidVersion: readiness.recommendedAndroidVersion,
            profile: settings.defaultProfile || "Gaming Phone",
            cpuCores: readiness.recommendedCpuCores,
            ramMb: readiness.recommendedRamMb,
            adbPort: 5555,
            adbEnabled: settings.defaultAdbEnabled,
            headless: settings.defaultHeadless,
            rootMode: "standard",
            imagePath: ""
          }
        });
      }

      const saved = await invoke<AppSettings>("save_app_settings", {
        settings: {
          ...settings,
          defaultAndroidVersion: readiness.recommendedAndroidVersion,
          firstRunCompleted: true
        }
      });
      setSettings(saved);
      setStatus(existing.length === 0 ? "First-run setup completed and the first Android instance was created." : "First-run setup completed.");
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusy(false);
    }
  };

  const check = (ready: boolean, readyText: string, missingText: string) => (
    <article>
      <span>{ready ? "Ready" : "Needs attention"}</span>
      <strong>{ready ? "✓" : "!"}</strong>
      <small>{ready ? readyText : missingText}</small>
    </article>
  );

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">First-Run Wizard</p><h3>Prepare this PC for NekoDroid</h3></div>
        <span className="pill pill-warn">Setup required</span>
      </div>

      {!readiness ? (
        <p className="muted">Scanning this PC…</p>
      ) : (
        <>
          <div className="stats">
            {check(
              readiness.virtualizationAvailable,
              "CPU virtualization acceleration is available.",
              readiness.virtualizationDetail
            )}
            {check(
              readiness.qemuFound,
              `${readiness.runtimeName} is available.`,
              readiness.runtimeName === "MSYS2 Runtime"
                ? "MSYS2 runtime is missing. Install it under C:\\msys64 or set MSYS2_ROOT."
                : "QEMU is missing from PATH."
            )}
            {check(
              readiness.adbFound,
              "Android platform-tools are available.",
              "ADB is missing from PATH."
            )}
            {check(
              readiness.ffmpegFound,
              "FFmpeg is available.",
              "FFmpeg is missing from PATH."
            )}
          </div>

          <div className="developer-grid">
            <div className="tool-column">
              <h4>Graphics</h4>
              <small>GPU: {readiness.gpuNames.length ? readiness.gpuNames.join(", ") : "not detected"}</small>
              <small>Vulkan: {readiness.vulkanAvailable ? "detected" : "not detected"}</small>
              <small>{readiness.vulkanDetail}</small>

              <h4>Disk</h4>
              <small>
                Free space: {readiness.freeDiskMb == null
                  ? "unknown"
                  : `${Math.round(readiness.freeDiskMb / 1024)} GB`}
              </small>

              <h4>Recommended starting point</h4>
              <small>Android {readiness.recommendedAndroidVersion}</small>
              <small>Renderer: {readiness.recommendedRenderer}</small>
              <small>{readiness.recommendedCpuCores} vCPU</small>
              <small>{Math.round(readiness.recommendedRamMb / 1024)} GB guest RAM</small>
            </div>

            <div className="tool-column">
              <div className="warning-box">
                The wizard does not bypass missing host virtualization or GPU support. Fix any host requirement shown above before expecting high-performance Android virtualization.
              </div>
              <button className="primary" disabled={busy || !readiness.virtualizationAvailable} onClick={finish}>
                {busy ? "Saving…" : "Use Recommendations & Finish"}
              </button>
              {status && <pre className="inline-output">{status}</pre>}
            </div>
          </div>
        </>
      )}
    </section>
  );
}
