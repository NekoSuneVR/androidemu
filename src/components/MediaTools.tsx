import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { FfmpegInfo, MediaResult } from "../types";

export default function MediaTools() {
  const [info, setInfo] = useState<FfmpegInfo | null>(null);
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [operation, setOperation] = useState<"video" | "audio" | "extract-audio" | "remux">("video");
  const [videoCodec, setVideoCodec] = useState("libx264");
  const [audioCodec, setAudioCodec] = useState("aac");
  const [width, setWidth] = useState(1920);
  const [height, setHeight] = useState(1080);
  const [fps, setFps] = useState(60);
  const [useResize, setUseResize] = useState(false);
  const [useFps, setUseFps] = useState(false);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState("FFmpeg job output will appear here.");

  useEffect(() => {
    invoke<FfmpegInfo>("get_ffmpeg_info")
      .then(setInfo)
      .catch(error => setResult(String(error)));
  }, []);

  const run = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      const response = await invoke<MediaResult>("run_media_job", {
        input,
        output,
        operation,
        videoCodec,
        audioCodec,
        width: useResize && operation === "video" ? width : null,
        height: useResize && operation === "video" ? height : null,
        fps: useFps && operation === "video" ? fps : null
      });

      setResult([
        response.success ? "SUCCESS" : `FAILED (exit ${response.exitCode ?? "unknown"})`,
        response.outputPath,
        response.stdout,
        response.stderr
      ].filter(Boolean).join("\n\n"));
    } catch (error) {
      setResult(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Media Tools</p><h3>FFmpeg conversion engine</h3></div>
        <span className={info?.found ? "pill" : "pill pill-warn"}>{info?.found ? "FFmpeg ready" : "FFmpeg missing"}</span>
      </div>

      <div className="developer-grid">
        <form className="tool-column" onSubmit={run}>
          <label>Input file
            <input value={input} placeholder="C:\Videos\input.mp4" onChange={e => setInput(e.target.value)} />
          </label>
          <label>Output file
            <input value={output} placeholder="C:\Videos\output.mp4" onChange={e => setOutput(e.target.value)} />
          </label>

          <label>Operation
            <select value={operation} onChange={e => setOperation(e.target.value as typeof operation)}>
              <option value="video">Video convert</option>
              <option value="audio">Audio convert</option>
              <option value="extract-audio">Extract audio</option>
              <option value="remux">Remux without re-encoding</option>
            </select>
          </label>

          {operation !== "remux" && (
            <>
              {operation === "video" && (
                <label>Video codec<input value={videoCodec} onChange={e => setVideoCodec(e.target.value)} /></label>
              )}
              <label>Audio codec<input value={audioCodec} onChange={e => setAudioCodec(e.target.value)} /></label>
            </>
          )}

          {operation === "video" && (
            <>
              <label className="checkbox-line">
                <input type="checkbox" checked={useResize} onChange={e => setUseResize(e.target.checked)} />
                Resize output
              </label>
              {useResize && (
                <div className="split-fields">
                  <label>Width<input type="number" min="1" max="8192" value={width} onChange={e => setWidth(Number(e.target.value))} /></label>
                  <label>Height<input type="number" min="1" max="8192" value={height} onChange={e => setHeight(Number(e.target.value))} /></label>
                </div>
              )}

              <label className="checkbox-line">
                <input type="checkbox" checked={useFps} onChange={e => setUseFps(e.target.checked)} />
                Change frame rate
              </label>
              {useFps && (
                <label>FPS<input type="number" min="1" max="240" value={fps} onChange={e => setFps(Number(e.target.value))} /></label>
              )}
            </>
          )}

          <button className="primary" disabled={busy || !info?.found || !input || !output}>
            {busy ? "Processing..." : "Run FFmpeg Job"}
          </button>

          <div className="warning-box">
            This page uses your installed FFmpeg build. Available codecs and hardware acceleration depend on that FFmpeg package and host GPU drivers.
          </div>
        </form>

        <div className="terminal">
          <div className="terminal-title">FFmpeg status</div>
          <pre>{info?.version || "FFmpeg not detected"}</pre>
          {info?.hwaccels?.length ? (
            <>
              <div className="terminal-title">Hardware acceleration advertised</div>
              <pre>{info.hwaccels.join("\n")}</pre>
            </>
          ) : null}
          <div className="terminal-title">Job output</div>
          <pre>{result}</pre>
        </div>
      </div>
    </section>
  );
}
