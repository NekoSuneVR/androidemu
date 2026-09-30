import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { FfmpegInfo, FfmpegSettings, MediaCodecCapabilityReport, MediaJobRequest, MediaJobStatus, MediaResult } from "../types";

export default function MediaTools() {
  const [info, setInfo] = useState<FfmpegInfo | null>(null);
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [operation, setOperation] = useState<"video" | "compress" | "audio" | "extract-audio" | "remux">("video");
  const [videoCodec, setVideoCodec] = useState("libx264");
  const [audioCodec, setAudioCodec] = useState("aac");
  const [width, setWidth] = useState(1920);
  const [height, setHeight] = useState(1080);
  const [fps, setFps] = useState(60);
  const [useResize, setUseResize] = useState(false);
  const [useFps, setUseFps] = useState(false);
  const [hardwareDecode, setHardwareDecode] = useState(false);
  const [busy, setBusy] = useState(false);
  const [batchLines, setBatchLines] = useState("");
  const [result, setResult] = useState("FFmpeg job output will appear here.");
  const [streamUrl,setStreamUrl]=useState("");
  const [streamRotate,setStreamRotate]=useState<"none"|"left"|"right"|"flip">("none");
  const [codecReport, setCodecReport] = useState<MediaCodecCapabilityReport | null>(null);
  const [ffmpegSettings,setFfmpegSettings]=useState<FfmpegSettings|null>(null);
  const [managedJobs,setManagedJobs]=useState<MediaJobStatus[]>([]);

  useEffect(() => {
    invoke<FfmpegInfo>("get_ffmpeg_info")
      .then(setInfo)
      .catch(error => setResult(String(error)));
    invoke<MediaCodecCapabilityReport>("get_media_codec_report").then(setCodecReport).catch(error => setResult(String(error)));
    invoke<FfmpegSettings>("get_ffmpeg_settings").then(setFfmpegSettings).catch(error=>setResult(String(error)));
  }, []);

  useEffect(()=>{
    const refresh=()=>invoke<MediaJobStatus[]>("list_media_jobs").then(setManagedJobs).catch(()=>{});
    refresh();const timer=window.setInterval(refresh,1000);return()=>window.clearInterval(timer);
  },[]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWindow().onDragDropEvent(event => {
      if (event.payload.type === "drop" && event.payload.paths.length > 0) {
        setInput(event.payload.paths[0]);
        setResult(`Dropped input: ${event.payload.paths[0]}`);
      }
    }).then(fn => { unlisten = fn; }).catch(error => setResult(String(error)));
    return () => unlisten?.();
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
        width: useResize && (operation === "video" || operation === "compress") ? width : null,
        height: useResize && (operation === "video" || operation === "compress") ? height : null,
        fps: useFps && (operation === "video" || operation === "compress") ? fps : null,
        hardwareDecode
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

  const runBatch = async () => {
    const rows = batchLines
      .split(/\r?\n/)
      .map(line => line.trim())
      .filter(Boolean);

    if (!rows.length) {
      setResult("Add at least one batch row using: input path | output path");
      return;
    }

    const jobs: MediaJobRequest[] = rows.map(row => {
      const [batchInput, batchOutput] = row.split("|").map(value => value?.trim());
      if (!batchInput || !batchOutput) {
        throw new Error(`Invalid batch row: ${row}`);
      }
      return {
        input: batchInput,
        output: batchOutput,
        operation,
        videoCodec,
        audioCodec,
        width: useResize && (operation === "video" || operation === "compress") ? width : null,
        height: useResize && (operation === "video" || operation === "compress") ? height : null,
        fps: useFps && (operation === "video" || operation === "compress") ? fps : null,
        hardwareDecode
      };
    });

    setBusy(true);
    try {
      const responses = await invoke<MediaResult[]>("run_media_batch", { jobs });
      setResult(responses.map((response, index) => [
        `Job ${index + 1}: ${response.success ? "SUCCESS" : "FAILED"}`,
        response.outputPath,
        response.stderr || response.stdout
      ].filter(Boolean).join("\n")).join("\n\n"));
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
              <option value="compress">Video compress</option>
              <option value="audio">Audio convert</option>
              <option value="extract-audio">Extract audio</option>
              <option value="remux">Remux without re-encoding</option>
            </select>
          </label>

          {operation !== "remux" && (
            <>
              {(operation === "video" || operation === "compress") && (
                <label>Video codec
                  <input list="video-codecs" value={videoCodec} onChange={e => setVideoCodec(e.target.value)} />
                  <datalist id="video-codecs">
                    <option value="libx264" />
                    <option value="libx265" />
                    <option value="libvpx" />
                    <option value="libvpx-vp9" />
                    <option value="libaom-av1" />
                    <option value="mpeg4" />
                    <option value="mpeg2video" />
                    {info?.hardwareEncoders?.map(codec => <option key={codec} value={codec} />)}
                  </datalist>
                </label>
              )}
              <label>Audio codec
                <input list="audio-codecs" value={audioCodec} onChange={e => setAudioCodec(e.target.value)} />
                <datalist id="audio-codecs">
                  <option value="aac" />
                  <option value="libmp3lame" />
                  <option value="libopus" />
                  <option value="libvorbis" />
                  <option value="flac" />
                  <option value="pcm_s16le" />
                  <option value="alac" />
                </datalist>
              </label>
            </>
          )}

          {(operation === "video" || operation === "compress") && (
            <>
              <label className="checkbox-line">
                <input type="checkbox" checked={hardwareDecode} onChange={e => setHardwareDecode(e.target.checked)} />
                Use FFmpeg hardware decode auto-detection
              </label>
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
          <button type="button" className="ghost" disabled={!info?.found||!input||!output} onClick={async()=>{
            try{
              const request:MediaJobRequest={input,output,operation,videoCodec,audioCodec,width:useResize&&(operation==="video"||operation==="compress")?width:null,height:useResize&&(operation==="video"||operation==="compress")?height:null,fps:useFps&&(operation==="video"||operation==="compress")?fps:null,hardwareDecode};
              const job=await invoke<MediaJobStatus>("start_media_job",{request});
              setResult(`Started managed FFmpeg job ${job.id}`);
            }catch(error){setResult(String(error));}
          }}>Run with progress / cancel</button>

          {ffmpegSettings && <div className="tool-group">
            <h4>FFmpeg manager</h4>
            <label>Custom FFmpeg path<input value={ffmpegSettings.customFfmpegPath} onChange={e=>setFfmpegSettings({...ffmpegSettings,customFfmpegPath:e.target.value})}/></label>
            <label>Custom FFprobe path<input value={ffmpegSettings.customFfprobePath} onChange={e=>setFfmpegSettings({...ffmpegSettings,customFfprobePath:e.target.value})}/></label>
            <label>Preferred hardware encoder<input list="managed-hwenc" value={ffmpegSettings.preferredHardwareEncoder} onChange={e=>setFfmpegSettings({...ffmpegSettings,preferredHardwareEncoder:e.target.value})}/><datalist id="managed-hwenc"><option value="auto"/>{info?.hardwareEncoders.map(v=><option key={v} value={v}/>)}</datalist></label>
            <label>Recording quality<select value={ffmpegSettings.recordingQuality} onChange={e=>setFfmpegSettings({...ffmpegSettings,recordingQuality:e.target.value as FfmpegSettings["recordingQuality"]})}>{["low","medium","high","lossless"].map(v=><option key={v}>{v}</option>)}</select></label>
            <label className="checkbox-line"><input type="checkbox" checked={ffmpegSettings.obsFriendly} onChange={e=>setFfmpegSettings({...ffmpegSettings,obsFriendly:e.target.checked})}/>OBS-friendly yuv420p + faststart output</label>
            <button type="button" className="ghost compact" onClick={async()=>{try{setFfmpegSettings(await invoke<FfmpegSettings>("save_ffmpeg_settings",{settings:ffmpegSettings}));setResult("FFmpeg manager settings saved.");}catch(error){setResult(String(error));}}}>Save FFmpeg settings</button>
          </div>}

          <div className="tool-group">
            <h4>RTMP / SRT streaming</h4>
            <label>Stream URL<input placeholder="rtmp://server/app/key or srt://host:port" value={streamUrl} onChange={e=>setStreamUrl(e.target.value)}/></label>
            <label>Rotation<select value={streamRotate} onChange={e=>setStreamRotate(e.target.value as typeof streamRotate)}><option value="none">none</option><option value="left">left</option><option value="right">right</option><option value="flip">flip</option></select></label>
            <button type="button" className="ghost compact" disabled={busy||!input||!streamUrl} onClick={async()=>{try{setBusy(true);const response=await invoke<MediaResult>("stream_media",{input,url:streamUrl,videoCodec,rotate:streamRotate});setResult([response.success?"STREAM ENDED":"STREAM FAILED",response.stderr||response.stdout].join("\n"));}catch(error){setResult(String(error));}finally{setBusy(false);}}}>Start stream</button>
          </div>

          <div className="tool-group">
            <h4>Batch queue</h4>
            <label>Jobs, one per line: input path | output path
              <textarea
                placeholder={"C:\\Videos\\a.mp4 | C:\\Videos\\a-out.mp4\nC:\\Videos\\b.mp4 | C:\\Videos\\b-out.mp4"}
                value={batchLines}
                onChange={e => setBatchLines(e.target.value)}
              />
            </label>
            <button type="button" className="ghost compact" disabled={busy || !batchLines.trim() || !info?.found} onClick={runBatch}>
              Run Batch Queue
            </button>
          </div>

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
          {info?.hardwareEncoders?.length ? (
            <>
              <div className="terminal-title">Hardware encoders</div>
              <pre>{info.hardwareEncoders.join("\n")}</pre>
            </>
          ) : null}
          {info?.hardwareDecoders?.length ? (
            <>
              <div className="terminal-title">Hardware decoders</div>
              <pre>{info.hardwareDecoders.join("\n")}</pre>
            </>
          ) : null}
          {info?.codecCapabilities?.length ? (
            <>
              <div className="terminal-title">Common codec capabilities</div>
              <pre>{info.codecCapabilities.join("\n")}</pre>
            </>
          ) : null}
          {codecReport && <>
            <div className="terminal-title">MediaCodec host bridge</div>
            <pre>{[
              ...codecReport.hardwareFamilies,
              ...codecReport.capabilities.map(c => `${c.codec}: decode=${c.decode} encode=${c.encode} decoder=${c.preferredDecoder ?? "-"} encoder=${c.preferredEncoder ?? "-"} softwareFallback=${c.softwareFallback}`)
            ].join("\n")}</pre>
          </>}
          <div className="terminal-title">Managed jobs</div>
          {managedJobs.map(job=><div key={job.id} className="instance-card"><strong>{job.status} · {job.progress.toFixed(1)}%</strong><small>{job.input} → {job.output}</small><small>{job.message}</small>{job.status==="running"&&<button className="danger compact" onClick={async()=>{await invoke("cancel_media_job",{id:job.id});setManagedJobs(await invoke<MediaJobStatus[]>("list_media_jobs"));}}>Cancel</button>}</div>)}
          <div className="terminal-title">Job output</div>
          <pre>{result}</pre>
        </div>
      </div>
    </section>
  );
}
