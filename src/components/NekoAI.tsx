import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AdbResult, AiCaptureResult, AiChatResult, AiGameState, AiSettings, AndroidInstance, SkillManifest } from "../types";

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
  const [skills, setSkills] = useState<SkillManifest[]>([]);
  const [skillJson, setSkillJson] = useState("");
  const [skillPath, setSkillPath] = useState("");
  const [gameState, setGameState] = useState<AiGameState | null>(null);
  const [capturePath,setCapturePath]=useState("nekodroid-ai-frame.png");
  const [audioPath,setAudioPath]=useState("");
  const [ttsText,setTtsText]=useState("Hello from NekoAI");
  const [roi,setRoi]=useState({x:0,y:0,width:640,height:480});

  const selectedInstance = instances.find(instance => instance.id === instanceId) ?? instances[0];

  useEffect(() => {
    if (!instanceId && instances[0]) setInstanceId(instances[0].id);
  }, [instances, instanceId]);

  useEffect(() => {
    invoke<AiSettings>("get_ai_settings")
      .then(setSettings)
      .catch(error => setOutput(String(error)));
    invoke<SkillManifest[]>("list_ai_skills")
      .then(setSkills)
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
              <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance} onClick={() => runAiAction({ kind:"joystick", centerX:x1, centerY:y1, dx:x2-x1, dy:y2-y1, durationMs })}>AI Joystick</button>
              <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance} onClick={() => runAiAction({ kind:"gamepad", button:"a" })}>AI Gamepad A</button>
              <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance} onClick={() => runAiAction({ kind:"multi-touch", points:[[x1,y1],[x2,y2]], durationMs })}>AI Multi-touch</button>
              <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance} onClick={() => runAiAction({ kind:"gyro", x:0, y:0, z:1 })}>AI Gyro Test</button>
              <button type="button" className="ghost compact" disabled={busy || !settings.enabled || !selectedInstance} onClick={() => runAiAction({ kind:"accelerometer", x:0, y:9.81, z:0 })}>AI Accelerometer Test</button>
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

          <div className="tool-group">
            <h4>Vision / OCR / voice</h4>
            <label>Capture path<input value={capturePath} onChange={e=>setCapturePath(e.target.value)}/></label>
            <div className="split-fields">
              <label>ROI X<input type="number" min="0" value={roi.x} onChange={e=>setRoi({...roi,x:Number(e.target.value)})}/></label>
              <label>ROI Y<input type="number" min="0" value={roi.y} onChange={e=>setRoi({...roi,y:Number(e.target.value)})}/></label>
              <label>ROI width<input type="number" min="1" value={roi.width} onChange={e=>setRoi({...roi,width:Number(e.target.value)})}/></label>
              <label>ROI height<input type="number" min="1" value={roi.height} onChange={e=>setRoi({...roi,height:Number(e.target.value)})}/></label>
            </div>
            <div className="button-row">
              <button type="button" className="ghost compact" disabled={!selectedInstance} onClick={async()=>{if(!selectedInstance)return;try{const r=await invoke<AiCaptureResult>("ai_capture_frame",{port:selectedInstance.adbPort,destination:capturePath,maxFps:settings.maxCaptureFps});setOutput(`Captured ${r.path}\nChanged: ${r.changed}\nSHA-256: ${r.sha256}`);}catch(error){setOutput(String(error));}}}>Capture frame</button>
              <button type="button" className="ghost compact" disabled={!selectedInstance} onClick={async()=>{if(!selectedInstance)return;try{const r=await invoke<AiCaptureResult>("ai_capture_roi",{port:selectedInstance.adbPort,destination:capturePath,...roi,maxFps:settings.maxCaptureFps});setOutput(`Captured ROI ${r.path}\nSHA-256: ${r.sha256}`);}catch(error){setOutput(String(error));}}}>Capture ROI</button>
              <button type="button" className="ghost compact" disabled={!capturePath} onClick={async()=>{try{setOutput(await invoke<string>("ai_ocr",{imagePath:capturePath}));}catch(error){setOutput(String(error));}}}>OCR image</button>
              <button type="button" className="ghost compact" disabled={!selectedInstance} onClick={async()=>{if(!selectedInstance)return;try{setOutput(await invoke<string>("ai_input_visualizer",{port:selectedInstance.adbPort}));}catch(error){setOutput(String(error));}}}>Input visualizer</button>
            </div>
            <label>Audio file for speech recognition<input value={audioPath} onChange={e=>setAudioPath(e.target.value)}/></label>
            <div className="button-row">
              <button type="button" className="ghost compact" disabled={!audioPath} onClick={async()=>{try{const text=await invoke<string>("ai_speech_to_text",{audioPath});setPrompt(text);setOutput(text);}catch(error){setOutput(String(error));}}}>Transcribe with Whisper CLI</button>
              <button type="button" className="primary compact" disabled={!audioPath||!settings.enabled} onClick={async()=>{try{setBusy(true);const text=await invoke<string>("ai_speech_to_text",{audioPath});setPrompt(text);const reply=await invoke<AiChatResult>("ai_chat",{prompt:text});setOutput([`Voice command: ${text}`,`Model: ${reply.model}`,reply.response].join("\n\n"));}catch(error){setOutput(String(error));}finally{setBusy(false);}}}>Voice command → NekoAI</button>
            </div>
            <label>TTS text<textarea value={ttsText} onChange={e=>setTtsText(e.target.value)}/></label>
            <button type="button" className="ghost compact" disabled={!ttsText.trim()} onClick={async()=>{try{setOutput(await invoke<string>("ai_tts",{text:ttsText}));}catch(error){setOutput(String(error));}}}>Speak with host TTS</button>
            <small className="muted">ADB screencap works without a visible emulator window, so headless/off-screen AI capture remains available. Capture FPS is limited by the AI setting above.</small>
          </div>

          <div className="tool-group">
            <h4>AI game skills</h4>
            <small className="muted">{skills.length} skill(s) loaded, including the built-in Generic Android skill.</small>
            <label>Skill JSON
              <textarea value={skillJson} placeholder='{"schemaVersion":1,"id":"my-game",...}' onChange={e => setSkillJson(e.target.value)} />
            </label>
            <div className="button-row">
              <button type="button" className="ghost compact" disabled={!skillJson.trim()} onClick={async () => {
                try {
                  const skill = JSON.parse(skillJson) as SkillManifest;
                  await invoke("save_ai_skill", { skill });
                  setSkills(await invoke<SkillManifest[]>("list_ai_skills"));
                  setOutput(`Saved AI skill ${skill.name}.`);
                } catch (error) { setOutput(String(error)); }
              }}>Save custom skill</button>
              <button type="button" className="ghost compact" disabled={!selectedInstance} onClick={async () => {
                try {
                  const state = await invoke<AiGameState>("get_ai_game_state", { port:selectedInstance?.adbPort });
                  setGameState(state);
                  setOutput(`Foreground package: ${state.packageName || "unknown"}\nOrientation: ${state.orientation}\nDisplay: ${state.displaySize}`);
                } catch (error) { setOutput(String(error)); }
              }}>Read game state</button>
            </div>
            <label>Import/export file path<input value={skillPath} onChange={e => setSkillPath(e.target.value)} /></label>
            <div className="button-row">
              <button type="button" className="ghost compact" disabled={!skillPath} onClick={async () => {
                try {
                  await invoke("import_ai_skill", { source:skillPath });
                  setSkills(await invoke<SkillManifest[]>("list_ai_skills"));
                  setOutput("Skill imported.");
                } catch (error) { setOutput(String(error)); }
              }}>Import skill</button>
              <button type="button" className="ghost compact" disabled={!skillPath || !skills.length} onClick={async () => {
                try {
                  await invoke("export_ai_skill", { id:skills[0].id, destination:skillPath });
                  setOutput("Skill exported.");
                } catch (error) { setOutput(String(error)); }
              }}>Export first skill</button>
            </div>
            {gameState && <small className="muted">{gameState.packageName} · {gameState.orientation} · {gameState.displaySize}</small>}
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
