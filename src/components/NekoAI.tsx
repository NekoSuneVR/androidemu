import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AiSettings } from "../types";

const defaults: AiSettings = {
  enabled: false,
  mode: "local",
  endpoint: "http://127.0.0.1:11434/v1",
  model: "qwen2.5:3b",
  visionModel: "qwen2.5vl:3b",
  detectorModel: "yolov8n.onnx",
  maxActionsPerMinute: 60,
  maxCaptureFps: 10
};

export default function NekoAI() {
  const [settings, setSettings] = useState<AiSettings>(defaults);
  const [output, setOutput] = useState("AI control is off by default.");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    invoke<AiSettings>("get_ai_settings")
      .then(setSettings)
      .catch(error => setOutput(String(error)));
  }, []);

  const save = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      const saved = await invoke<AiSettings>("save_ai_settings", { settings });
      setSettings(saved);
      setOutput(saved.enabled ? "NekoAI settings saved and AI is enabled." : "NekoAI settings saved. AI remains disabled.");
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const emergencyStop = async () => {
    setBusy(true);
    try {
      const stopped = await invoke<AiSettings>("ai_emergency_stop");
      setSettings(stopped);
      setOutput("Emergency stop applied. AI control is disabled.");
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">NekoAI</p><h3>AI control foundation</h3></div>
        <span className={settings.enabled ? "pill" : "pill pill-warn"}>{settings.enabled ? "Enabled" : "Disabled"}</span>
      </div>

      <div className="developer-grid">
        <form className="tool-column" onSubmit={save}>
          <label className="checkbox-line">
            <input
              type="checkbox"
              checked={settings.enabled}
              onChange={e => setSettings({...settings, enabled:e.target.checked})}
            />
            Enable AI control
          </label>

          <label>AI mode
            <select value={settings.mode} onChange={e => setSettings({...settings, mode:e.target.value as AiSettings["mode"]})}>
              <option value="local">Local AI</option>
              <option value="ollama">Remote Ollama</option>
              <option value="openai-compatible">OpenAI-compatible endpoint</option>
            </select>
          </label>

          <label>Endpoint
            <input
              value={settings.endpoint}
              disabled={settings.mode === "local"}
              placeholder="https://example.com/v1"
              onChange={e => setSettings({...settings, endpoint:e.target.value})}
            />
          </label>

          <label>Primary model<input value={settings.model} onChange={e => setSettings({...settings, model:e.target.value})} /></label>
          <label>Vision model<input value={settings.visionModel} onChange={e => setSettings({...settings, visionModel:e.target.value})} /></label>
          <label>Object detector<input value={settings.detectorModel} onChange={e => setSettings({...settings, detectorModel:e.target.value})} /></label>

          <div className="split-fields">
            <label>Max actions / minute
              <input type="number" min="1" max="600" value={settings.maxActionsPerMinute} onChange={e => setSettings({...settings, maxActionsPerMinute:Number(e.target.value)})} />
            </label>
            <label>Max capture FPS
              <input type="number" min="1" max="120" value={settings.maxCaptureFps} onChange={e => setSettings({...settings, maxCaptureFps:Number(e.target.value)})} />
            </label>
          </div>

          <div className="button-row">
            <button className="primary compact" disabled={busy}>{busy ? "Saving..." : "Save AI Settings"}</button>
            <button type="button" className="danger compact" disabled={busy} onClick={emergencyStop}>Emergency Stop</button>
          </div>

          <div className="warning-box">
            Enabling AI here only permits the subsystem to operate. Game-control execution remains permission-gated and should use NekoDroid virtual Android input rather than the host OS cursor.
          </div>
        </form>

        <div className="terminal">
          <div className="terminal-title">NekoAI status</div>
          <pre>{output}</pre>
        </div>
      </div>
    </section>
  );
}
