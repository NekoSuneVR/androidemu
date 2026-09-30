import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SystemReadiness } from "../types";

export default function SystemReadinessPanel() {
  const [readiness, setReadiness] = useState<SystemReadiness | null>(null);
  const [error, setError] = useState("");

  const refresh = () => {
    invoke<SystemReadiness>("get_system_readiness")
      .then(value => {
        setReadiness(value);
        setError("");
      })
      .catch(value => setError(String(value)));
  };

  useEffect(refresh, []);

  if (error) {
    return (
      <section className="panel">
        <div className="panel-heading">
          <div><p className="eyebrow">System Readiness</p><h3>Host diagnostics</h3></div>
          <span className="pill pill-warn">Unavailable</span>
        </div>
        <pre className="inline-output">{error}</pre>
      </section>
    );
  }

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">System Readiness</p><h3>Virtualization and media capability scan</h3></div>
        <button className="ghost compact" onClick={refresh}>Rescan</button>
      </div>

      {!readiness ? <p className="muted">Scanning host capabilities…</p> : (
        <>
          <div className="stats">
            <article>
              <span>CPU</span>
              <strong>{readiness.logicalCores} threads</strong>
              <small>{readiness.cpuModel}</small>
            </article>
            <article>
              <span>Virtualization</span>
              <strong>{readiness.virtualizationAvailable ? "Ready" : "Needs attention"}</strong>
              <small>{readiness.virtualizationDetail}</small>
            </article>
            <article>
              <span>Vulkan</span>
              <strong>{readiness.vulkanAvailable ? "Detected" : "Not detected"}</strong>
              <small>{readiness.vulkanDetail}</small>
            </article>
            <article>
              <span>OpenGL</span>
              <strong>{readiness.openglAvailable ? "Detected" : "Not detected"}</strong>
              <small>{readiness.openglDetail}</small>
            </article>
            <article>
              <span>DirectX</span>
              <strong>{readiness.directxAvailable ? "Detected" : "Not detected"}</strong>
              <small>{readiness.directxDetail}</small>
            </article>
            <article>
              <span>Memory</span>
              <strong>{readiness.totalMemoryMb ? `${Math.round(readiness.totalMemoryMb / 1024)} GB` : "Unknown"}</strong>
              <small>Recommended guest: {Math.round(readiness.recommendedRamMb / 1024)} GB</small>
            </article>
          </div>

          <div className="developer-grid">
            <div className="tool-column">
              <h4>Detected GPUs</h4>
              {readiness.gpuNames.length ? readiness.gpuNames.map(name => <small key={name}>{name}</small>) : <small className="muted">No GPU name detected.</small>}

              <h4>Core tools</h4>
              <small>{readiness.runtimeName}: {readiness.qemuFound ? "detected" : "missing"}</small>
              <small>ADB: {readiness.adbFound ? "detected" : "missing"}</small>
              <small>FFmpeg: {readiness.ffmpegFound ? "detected" : "missing"}</small>

              <h4>Recommendations</h4>
              <small>Android {readiness.recommendedAndroidVersion}</small>
              <small>Renderer: {readiness.recommendedRenderer}</small>
              <small>{readiness.recommendedCpuCores} vCPU</small>
              <small>{Math.round(readiness.recommendedRamMb / 1024)} GB RAM</small>
            </div>

            <div className="terminal">
              <div className="terminal-title">Hardware encoders detected by FFmpeg</div>
              <pre>{readiness.hardwareEncoders.length ? readiness.hardwareEncoders.join("\n") : "No hardware encoders detected."}</pre>
            </div>
          </div>
        </>
      )}
    </section>
  );
}
