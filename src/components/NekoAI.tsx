import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AdbResult, AiChatResult, AiSettings, AndroidInstance } from "../types";

const defaults: AiSettings = {
  enabled: false,
  mode: "local",
  endpoint: "http://127.0.0.1:11434/v1",
  model: "qwen2.5:3b",
  visionModel: "qwen2.5vl:3b",
  detectorModel: "yolov8n.onnx",
  controlMode: "manual",
  helperMode: "ui",
  enforcePackageAllowlist: false,
  allowedPackages: [],
  maxActionsPerMinute: 60,
  maxCaptureFps: 10
};

export default function NekoAI({ instances }: { instances: AndroidInstance[] }) {
  const [settings, setSettings] = useState<AiSettings>(defaults);
  const [output, setOutput] = useState("AI control is off by default.");
  const [prompt, setPrompt] = useState("");
  const [logs, setLogs] = useState("");
  const [instanceId, setInstanceId] = useState(instances[0]?.id ?? "");
  const [x1, setX1] = useState(540);
  const [y1, setY1] = useState(1200);
  const [x2, setX2] = useState(900);
  const [y2, setY2] = useState(1200);
  const [durationMs, setDurationMs] = useState(600);
  const [keycode, setKeycode] = useState("KEYCODE_ENTER");
  const [inputText, setInputText] = useState("hello");
  const [queueJson, setQueueJson] = useState('[{"kind":"tap","x":540,"y":1200},{"kind":"hold","x":540,"y":1200,"durationMs":500}]');
  const [busy, setBusy] = useState(false);

  const selectedInstance = instances.find(instance => instance.id === instanceId) ?? instances[0];

  useEffect(() => {
    if (!instanceId && instances[0]) setInstanceId(instances[0].id);
  }, [instances, instanceId]);

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

  const runAiAction = async (action: Record<string, unknown>) => {
    if (!selectedInstance) {
      setOutput("Create an Android instance before using AI controls.");
      return;
    }
    setBusy(true);
    try {
      const result = await invoke<AdbResult>("ai_execute_action", {
        port: selectedInstance.adbPort,
        action
      });
      setOutput([
        result.success ? "AI ACTION SUCCESS" : `AI ACTION FAILED (exit ${result.exitCode ?? "unknown"})`,
        result.stdout,
        result.stderr
      ].filter(Boolean).join("\n\n"));
      setLogs(await invoke<string>("get_ai_logs"));
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const runAiQueue = async () => {
    if (!selectedInstance) {
      setOutput("Create an Android instance before using the AI queue.");
      return;
    }
    setBusy(true);
    try {
      const actions = JSON.parse(queueJson);
      if (!Array.isArray(actions)) throw new Error("Queue JSON must be an array of actions.");
      const results = await invoke<AdbResult[]>("ai_execute_actions", {
        port: selectedInstance.adbPort,
        actions
      });
      setOutput(results.map((result, index) => [
        `Action ${index + 1}: ${result.success ? "SUCCESS" : "FAILED"}`,
        result.stdout,
        result.stderr
      ].filter(Boolean).join("\n")).join("\n\n"));
      setLogs(await invoke<string>("get_ai_logs"));
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const cancelAiQueue = async () => {
    try {
      await invoke("ai_cancel_actions");
      setOutput("AI action queue cancellation requested.");
    } catch (error) {
      setOutput(String(error));
    }
  };

  const sendPrompt = async () => {
    if (!prompt.trim()) return;
    setBusy(true);
    try {
      const result = await invoke<AiChatResult>("ai_chat", { prompt });
      setOutput([
        `Model: ${result.model}`,
        `Endpoint: ${result.endpoint}`,
        "",
        result.response
      ].join("\n"));
      setPrompt("");
      const currentLogs = await invoke<string>("get_ai_logs");
      setLogs(currentLogs);
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const loadLogs = async () => {
    try {
      setLogs(await invoke<string>("get_ai_logs"));
    } catch (error) {
      setLogs(String(error));
    }
  };

  const clearLogs = async () => {
    try {
      await invoke("clear_ai_logs");
      setLogs("");
    } catch (error) {
      setLogs(String(error));
    }
  };

  const emergencyStop = async () => {
    setBusy(true);
    try {
      await invoke("ai_cancel_actions");
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

          <label>Control mode
            <select value={settings.controlMode} onChange={e => setSettings({...settings, controlMode:e.target.value as AiSettings["controlMode"]})}>
              <option value="manual">Manual (chat only)</option>
              <option value="assistant">Assistant (short action queues)</option>
              <option value="accessibility">Accessibility</option>
              <option value="full-automation">Full automation</option>
            </select>
          </label>

          <label>Helper mode
            <select value={settings.helperMode} onChange={e => setSettings({...settings, helperMode:e.target.value as AiSettings["helperMode"]})}>
              <option value="inventory">Inventory helper</option>
              <option value="quest">Quest helper</option>
              <option value="ui">UI helper</option>
              <option value="repetitive-task">Repetitive-task helper</option>
            </select>
          </label>

          <label className="checkbox-line">
            <input type="checkbox" checked={settings.enforcePackageAllowlist} onChange={e => setSettings({...settings,enforcePackageAllowlist:e.target.checked})} />
            Restrict AI controls to approved Android packages
          </label>
          <label>Allowed packages, one per line
            <textarea
              value={settings.allowedPackages.join("\n")}
              placeholder={"com.example.game\ncom.example.accessibilityapp"}
              onChange={e => setSettings({...settings,allowedPackages:e.target.value.split(/\r?\n/).map(v => v.trim()).filter(Boolean)})}
            />
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

          <div className="tool-group">
            <h4>Chat / connection test</h4>
            <label>Prompt
              <textarea
                value={prompt}
                placeholder="Ask the configured NekoAI model something..."
                onChange={e => setPrompt(e.target.value)}
              />
            </label>
            <button
              type="button"
              className="ghost compact"
              disabled={busy || !settings.enabled || !prompt.trim()}
              onClick={sendPrompt}
            >
              Send to NekoAI
            </button>
          </div>

          <div className="tool-group">
            <h4>AI virtual Android controls</h4>
            <label>Instance
              <select value={selectedInstance?.id ?? ""} onChange={e => setInstanceId(e.target.value)}>
                {instances.map(instance => <option key={instance.id} value={instance.id}>{instance.name}</option>)}
              </select>
            </label>

            <div className="split-fields">
              <label>X1<input type="number" min="0" max="32767" value={x1} onChange={e => setX1(Number(e.target.value))} /></label>
              <label>Y1<input type="number" min="0" max="32767" value={y1} onChange={e => setY1(Number(e.target.value))} /></label>
            </div>
            <div className="split-fields">
              <label>X2<input type="number" min="0" max="32767" value={x2} onChange={e => setX2(Number(e.target.value))} /></label>
              <label>Y2<input type="number" min="0" max="32767" value={y2} onChange={e => setY2(Number(e.target.value))} /></label>
            </div>
            <label>Duration ms<input type="number" min="50" max="5000" value={durationMs} onChange={e => setDurationMs(Number(e.target.value))} /></label>

            <div className="button-row">
              <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance} onClick={() => runAiAction({ kind:"tap", x:x1, y:y1 })}>AI Tap</button>
              <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance} onClick={() => runAiAction({ kind:"hold", x:x1, y:y1, durationMs })}>AI Hold</button>
              <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance} onClick={() => runAiAction({ kind:"swipe", x1, y1, x2, y2, durationMs })}>AI Swipe</button>
              <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance} onClick={() => runAiAction({ kind:"drag", x1, y1, x2, y2, durationMs })}>AI Drag</button>
            </div>

            <label>Android keycode<input value={keycode} onChange={e => setKeycode(e.target.value)} /></label>
            <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance || !keycode} onClick={() => runAiAction({ kind:"key", keycode })}>AI Key</button>

            <label>Text<input value={inputText} onChange={e => setInputText(e.target.value)} /></label>
            <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance || !inputText} onClick={() => runAiAction({ kind:"text", text:inputText })}>AI Text</button>

            <h4>AI action queue</h4>
            <label>Action JSON array
              <textarea value={queueJson} onChange={e => setQueueJson(e.target.value)} />
            </label>
            <div className="button-row">
              <button type="button" className="primary compact" disabled={busy || !settings.enabled || !selectedInstance || !queueJson.trim()} onClick={runAiQueue}>Run Queue</button>
              <button type="button" className="danger compact" onClick={cancelAiQueue}>Cancel Queue</button>
            </div>
          </div>

          <div className="warning-box">
            Enabling AI here only permits the subsystem to operate. Game-control execution uses NekoDroid virtual Android input directly and never moves the host OS mouse.
          </div>
        </form>

        <div className="terminal">
          <div className="terminal-title">NekoAI response</div>
          <pre>{output}</pre>
          <div className="button-row">
            <button className="ghost compact" onClick={loadLogs}>Load AI Logs</button>
            <button className="danger compact" onClick={clearLogs}>Clear AI Logs</button>
          </div>
          <div className="terminal-title">AI logs</div>
          <pre>{logs || "No AI log loaded."}</pre>
        </div>
      </div>
    </section>
  );
}
