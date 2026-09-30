import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AndroidInstance, DeviceProfile } from "../types";

type Props = {
  instances: AndroidInstance[];
  profiles: DeviceProfile[];
};

type Invite = {
  sessionId: string;
  hostToken: string;
  inviteCode: string;
  inviteUrl: string;
  expiresAt: string;
  nodeUrl: string;
};

type ViewerRequest = {
  viewerId: string;
  userAgent?: string;
  approved?: boolean;
  trusted?: boolean;
};

type RemoteSettings = {
  nodeUrl: string;
  nodeSecret: string;
  fallbackNodeUrls: string;
  unattendedTrusted: boolean;
  instanceId: string;
  control: boolean;
  clipboard: boolean;
  fileTransfer: boolean;
  gamepad: boolean;
  adaptiveBitrate: boolean;
  fpsPreset: 30|60|90|120;
  directFramebuffer: boolean;
  ttlSeconds: number;
};

const defaultSettings: RemoteSettings = {
  nodeUrl: "http://localhost:8096",
  nodeSecret: "",
  fallbackNodeUrls: "",
  unattendedTrusted: false,
  instanceId: "",
  control: true,
  clipboard: false,
  fileTransfer: false,
  gamepad: true,
  adaptiveBitrate: true,
  fpsPreset: 60,
  directFramebuffer: false,
  ttlSeconds: 900
};

export default function RemoteAccess({ instances, profiles }: Props) {
  const [settings, setSettings] = useState<RemoteSettings>(defaultSettings);
  const [invite, setInvite] = useState<Invite | null>(null);
  const [viewers, setViewers] = useState<ViewerRequest[]>([]);
  const [status, setStatus] = useState("No remote session active.");
  const [busy, setBusy] = useState(false);
  const socketRef = useRef<WebSocket | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const peersRef = useRef<Map<string, RTCPeerConnection>>(new Map());
  const pointerState = useRef<Map<string, { x: number; y: number; at: number }>>(new Map());
  const framebufferTimerRef=useRef<number|null>(null);

  const selected = useMemo(
    () => instances.find(instance => instance.id === settings.instanceId) ?? instances[0],
    [instances, settings.instanceId]
  );

  useEffect(() => {
    if (!settings.instanceId && instances[0]) {
      setSettings(current => ({ ...current, instanceId: instances[0].id }));
    }
  }, [instances, settings.instanceId]);

  useEffect(() => () => {
    socketRef.current?.close();
    streamRef.current?.getTracks().forEach(track => track.stop());
    if(framebufferTimerRef.current) window.clearInterval(framebufferTimerRef.current);
    peersRef.current.forEach(peer => peer.close());
  }, []);

  const createInvite = async () => {
    if (!selected) return setStatus("Create an Android instance first.");
    if (!settings.nodeUrl.trim() || !settings.nodeSecret) {
      return setStatus("Remote node URL and node secret are required.");
    }

    setBusy(true);
    try {
      const nodes=[settings.nodeUrl,...settings.fallbackNodeUrls.split(/\r?\n|,/).map(v=>v.trim()).filter(Boolean)]
        .map(v=>v.replace(/\/$/,""))
        .filter((v,i,a)=>a.indexOf(v)===i);
      let lastError="";
      for (const node of nodes) {
        try {
          const response = await fetch(`${node}/api/sessions`, {
            method: "POST",
            headers: {"content-type":"application/json",authorization:`Bearer ${settings.nodeSecret}`},
            body: JSON.stringify({
              name:selected.name, instanceId:selected.id, control:settings.control,
              clipboard:settings.clipboard, fileTransfer:settings.fileTransfer, gamepad:settings.gamepad,
              adaptiveBitrate:settings.adaptiveBitrate, fpsPreset:settings.fpsPreset,
              unattendedTrusted:settings.unattendedTrusted, ttlSeconds:settings.ttlSeconds
            })
          });
          const body=await response.json();
          if(!response.ok) throw new Error(body.error||`Remote node returned ${response.status}`);
          const created={...(body as Omit<Invite,"nodeUrl">),nodeUrl:node};
          setInvite(created);setViewers([]);setStatus(`Invite created on ${node}. Waiting for viewers.`);
          connectHostSocket(node,created);
          return;
        } catch(error) {
          lastError=String(error);
        }
      }
      throw new Error(`All signalling nodes failed. ${lastError}`);
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusy(false);
    }
  };

  const connectHostSocket = (node: string, created: Invite) => {
    socketRef.current?.close();
    const wsUrl = node.replace(/^http:/, "ws:").replace(/^https:/, "wss:") + "/ws";
    const ws = new WebSocket(wsUrl);
    socketRef.current = ws;

    ws.addEventListener("open", () => {
      ws.send(JSON.stringify({
        type: "host-auth",
        sessionId: created.sessionId,
        hostToken: created.hostToken
      }));
    });

    ws.addEventListener("message", event => {
      const message = JSON.parse(event.data);
      handleNodeMessage(message, node);
    });

    ws.addEventListener("close", () => {
      if (invite) setStatus("Remote node connection closed.");
    });
  };

  const handleNodeMessage = async (message: any, node: string) => {
    if (message.type === "host-ready") {
      setStatus("Remote node connected. Waiting for viewer approval requests.");
      return;
    }

    if (message.type === "viewer-request") {
      setViewers(current => current.some(v => v.viewerId === message.viewerId)
        ? current
        : [...current, { viewerId: message.viewerId, userAgent: message.userAgent, trusted:Boolean(message.trusted), approved:Boolean(message.approved) }]);
      if (message.approved && message.trusted && settings.unattendedTrusted) {
        window.setTimeout(() => approveViewer(message.viewerId), 0);
      }
      return;
    }

    if (message.type === "viewer-left") {
      closePeer(message.viewerId);
      setViewers(current => current.filter(v => v.viewerId !== message.viewerId));
      return;
    }

    if (message.type === "webrtc-answer") {
      const peer = peersRef.current.get(message.viewerId);
      if (peer) await peer.setRemoteDescription(message.sdp);
      return;
    }

    if (message.type === "ice-candidate") {
      const peer = peersRef.current.get(message.viewerId);
      if (peer && message.candidate) await peer.addIceCandidate(message.candidate);
      return;
    }

    if (message.type === "relay-data" && message.payload) {
      if (message.payload.kind === "clipboard" && settings.clipboard && selected) {
        await invoke("adb_clipboard_set", { port:selected.adbPort, text:String(message.payload.text ?? "") });
        return;
      }
      if (settings.control) {
        await applyControl(message.viewerId, message.payload);
      }
      return;
    }

    if (["session-expired", "session-revoked"].includes(message.type)) {
      await endSession(false);
    }
  };

  const decodePpmToCanvas = (base64:string,canvas:HTMLCanvasElement) => {
    const raw=atob(base64); const bytes=new Uint8Array(raw.length);
    for(let i=0;i<raw.length;i++) bytes[i]=raw.charCodeAt(i);
    let pos=0; const token=()=>{while(pos<bytes.length&&String.fromCharCode(bytes[pos]).trim()==="")pos++; let s=""; while(pos<bytes.length&&!/\s/.test(String.fromCharCode(bytes[pos])))s+=String.fromCharCode(bytes[pos++]); return s;};
    if(token()!=="P6") throw new Error("Unsupported framebuffer format");
    const width=Number(token()),height=Number(token()),max=Number(token()); while(pos<bytes.length&&/\s/.test(String.fromCharCode(bytes[pos])))pos++;
    if(!width||!height||max!==255) throw new Error("Invalid PPM framebuffer");
    canvas.width=width;canvas.height=height;const ctx=canvas.getContext("2d");if(!ctx)throw new Error("Canvas unavailable");
    const image=ctx.createImageData(width,height);let src=pos,dst=0;
    while(src+2<bytes.length&&dst<image.data.length){image.data[dst++]=bytes[src++];image.data[dst++]=bytes[src++];image.data[dst++]=bytes[src++];image.data[dst++]=255;}
    ctx.putImageData(image,0,0);
  };

  const ensureCapture = async () => {
    if (streamRef.current) return streamRef.current;
    if(settings.directFramebuffer){
      if(!selected||selected.status!=="running") throw new Error("Direct framebuffer streaming requires a running instance.");
      const profile=profiles.find(p=>p.name===selected.profile);
      const canvas=document.createElement("canvas");
      canvas.width=profile?.width??1080;canvas.height=profile?.height??2400;
      const refresh=async()=>{try{const frame=await invoke<string>("capture_instance_framebuffer_base64",{id:selected.id});decodePpmToCanvas(frame,canvas);}catch(error){setStatus(`Framebuffer capture: ${String(error)}`);}};
      await refresh();
      framebufferTimerRef.current=window.setInterval(refresh,Math.max(50,Math.round(1000/settings.fpsPreset)));
      const stream=(canvas as HTMLCanvasElement & {captureStream:(fps:number)=>MediaStream}).captureStream(settings.fpsPreset);
      streamRef.current=stream;
      return stream;
    }
    if (!navigator.mediaDevices?.getDisplayMedia) {
      throw new Error("Screen capture is unavailable in this WebView. Use a current Tauri/WebView2 build.");
    }
    const profile=selected?profiles.find(p=>p.name===selected.profile):undefined;
    const stream = await navigator.mediaDevices.getDisplayMedia({
      video: { frameRate: settings.fpsPreset, width:profile?.width, height:profile?.height },
      audio: true
    });
    const videoTrack = stream.getVideoTracks()[0];
    if (videoTrack) {
      await videoTrack.applyConstraints({ frameRate: settings.fpsPreset, width:profile?.width, height:profile?.height }).catch(() => {});
    }
    streamRef.current = stream;
    stream.getVideoTracks()[0]?.addEventListener("ended", () => {
      setStatus("Screen sharing stopped locally.");
      streamRef.current = null;
      peersRef.current.forEach(peer => peer.close());
      peersRef.current.clear();
    });
    return stream;
  };

  const approveViewer = async (viewerId: string) => {
    const ws = socketRef.current;
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    try {
      if (settings.control && selected) {
        const connected = await invoke<any>("adb_connect", { port: selected.adbPort });
        if (connected && connected.success === false) {
          throw new Error(connected.stderr || connected.stdout || "Unable to connect ADB for remote control");
        }
      }

      const stream = await ensureCapture();
      const activeNode=invite?.nodeUrl ?? settings.nodeUrl.replace(/\/$/, "");
      const configResponse = await fetch(`${activeNode}/api/config`, {
        headers: { authorization: `Bearer ${settings.nodeSecret}` }
      });
      const config = configResponse.ok ? await configResponse.json() : { iceServers: [] };

      const peer = new RTCPeerConnection({ iceServers: config.iceServers || [] });
      peersRef.current.set(viewerId, peer);

      stream.getTracks().forEach(track => peer.addTrack(track, stream));
      if (settings.adaptiveBitrate) {
        for (const sender of peer.getSenders()) {
          if (sender.track?.kind !== "video") continue;
          const params = sender.getParameters();
          params.encodings = params.encodings?.length ? params.encodings : [{}];
          params.encodings[0].maxBitrate = settings.fpsPreset >= 90 ? 12_000_000 : settings.fpsPreset >= 60 ? 8_000_000 : 4_000_000;
          await sender.setParameters(params).catch(() => {});
        }
      }
      const statsTimer = window.setInterval(async () => {
        if (peer.connectionState === "closed") return window.clearInterval(statsTimer);
        const stats = await peer.getStats().catch(() => null);
        if (!stats) return;
        let inbound = 0, outbound = 0, rtt = "";
        stats.forEach(report => {
          if (report.type === "outbound-rtp" && report.kind === "video") outbound = Number(report.bytesSent || 0);
          if (report.type === "inbound-rtp" && report.kind === "video") inbound = Number(report.bytesReceived || 0);
          if (report.type === "candidate-pair" && report.state === "succeeded" && report.currentRoundTripTime != null) rtt = `${Math.round(Number(report.currentRoundTripTime)*1000)}ms`;
        });
        setStatus(`Streaming · FPS ${settings.fpsPreset} · RTT ${rtt || "n/a"} · tx ${Math.round(outbound/1024)}KB · rx ${Math.round(inbound/1024)}KB`);
      }, 3000);

      if (settings.fileTransfer) {
        const fileChannel=peer.createDataChannel("file",{ordered:true});
        fileChannel.addEventListener("message",async event=>{
          try{
            const payload=JSON.parse(String(event.data));
            if(payload.kind!=="file"||!selected)return;
            setStatus(`Receiving encrypted file ${payload.name}...`);
            const message=await invoke<string>("receive_remote_file",{port:selected.adbPort,name:String(payload.name||"remote.bin"),dataBase64:String(payload.data||""),destination:"/sdcard/Download/"});
            setStatus(message);
          }catch(error){setStatus(`Remote file transfer failed: ${String(error)}`);}
        });
      }

      if (settings.control) {
        const channel = peer.createDataChannel("control", { ordered: true });
        channel.addEventListener("message", event => {
          try {
            applyControl(viewerId, JSON.parse(event.data));
          } catch {
            // Ignore malformed remote control packets.
          }
        });
      }

      peer.addEventListener("icecandidate", event => {
        if (event.candidate && ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({
            type: "ice-candidate",
            viewerId,
            candidate: event.candidate
          }));
        }
      });

      ws.send(JSON.stringify({ type: "viewer-approve", viewerId }));
      setViewers(current => current.map(v => v.viewerId === viewerId ? { ...v, approved: true } : v));

      const offer = await peer.createOffer();
      await peer.setLocalDescription(offer);
      ws.send(JSON.stringify({
        type: "webrtc-offer",
        viewerId,
        sdp: peer.localDescription
      }));
      setStatus("Viewer approved. Streaming the selected screen/window.");
    } catch (error) {
      setStatus(String(error));
    }
  };

  const denyViewer = (viewerId: string) => {
    socketRef.current?.send(JSON.stringify({ type: "viewer-deny", viewerId }));
    closePeer(viewerId);
    setViewers(current => current.filter(v => v.viewerId !== viewerId));
  };

  const revokeViewer = (viewerId: string) => {
    socketRef.current?.send(JSON.stringify({ type: "viewer-revoke", viewerId }));
    closePeer(viewerId);
    setViewers(current => current.filter(v => v.viewerId !== viewerId));
  };

  const closePeer = (viewerId: string) => {
    peersRef.current.get(viewerId)?.close();
    peersRef.current.delete(viewerId);
    pointerState.current.delete(viewerId);
  };

  const applyControl = async (viewerId: string, payload: any) => {
    if (!settings.control || !selected) return;
    const profile = profiles.find(p => p.name === selected.profile);
    const width = profile?.width ?? 1080;
    const height = profile?.height ?? 2400;

    if (payload.kind === "pointer") {
      const x = Math.round(Math.max(0, Math.min(1, Number(payload.x))) * (width - 1));
      const y = Math.round(Math.max(0, Math.min(1, Number(payload.y))) * (height - 1));

      if (payload.action === "down") {
        pointerState.current.set(viewerId, { x, y, at: Date.now() });
        return;
      }

      if (payload.action === "up") {
        const start = pointerState.current.get(viewerId);
        pointerState.current.delete(viewerId);
        if (!start) {
          await invoke("adb_input_tap", { port: selected.adbPort, x, y });
          return;
        }

        const distance = Math.hypot(x - start.x, y - start.y);
        if (distance < 12) {
          await invoke("adb_input_tap", { port: selected.adbPort, x, y });
        } else {
          const duration = Math.max(80, Math.min(1200, Date.now() - start.at));
          await invoke("adb_input_swipe", {
            port: selected.adbPort,
            x1: start.x,
            y1: start.y,
            x2: x,
            y2: y,
            durationMs: duration
          });
        }
      }
      return;
    }

    if (payload.kind === "gamepad" && settings.gamepad) {
      const pressed = Array.isArray(payload.buttons) ? payload.buttons : [];
      const mapping: Record<number,string> = {0:"KEYCODE_BUTTON_A",1:"KEYCODE_BUTTON_B",2:"KEYCODE_BUTTON_X",3:"KEYCODE_BUTTON_Y",12:"KEYCODE_DPAD_UP",13:"KEYCODE_DPAD_DOWN",14:"KEYCODE_DPAD_LEFT",15:"KEYCODE_DPAD_RIGHT"};
      for (const index of pressed) {
        const keycode=mapping[Number(index)];
        if (keycode) await invoke("adb_input_keyevent",{port:selected.adbPort,keycode});
      }
      return;
    }

    if (payload.kind === "android-key") {
      const keys: Record<string, string> = {
        BACK: "KEYCODE_BACK",
        HOME: "KEYCODE_HOME",
        APP_SWITCH: "KEYCODE_APP_SWITCH"
      };
      const key = keys[String(payload.key)];
      if (key) await invoke("adb_input_keyevent", { port: selected.adbPort, keycode: key });
      return;
    }

    if (payload.kind === "key" && payload.action === "down") {
      const key = String(payload.key || "");
      const keyMap: Record<string, string> = {
        Enter: "KEYCODE_ENTER",
        Escape: "KEYCODE_BACK",
        Backspace: "KEYCODE_DEL",
        ArrowUp: "KEYCODE_DPAD_UP",
        ArrowDown: "KEYCODE_DPAD_DOWN",
        ArrowLeft: "KEYCODE_DPAD_LEFT",
        ArrowRight: "KEYCODE_DPAD_RIGHT"
      };
      if (keyMap[key]) {
        await invoke("adb_input_keyevent", { port: selected.adbPort, keycode: keyMap[key] });
      } else if (key.length === 1 && /^[ -~]$/.test(key)) {
        await invoke("adb_input_text", { port: selected.adbPort, text: key });
      }
    }
  };


  const endSession = async (revokeOnServer = true) => {
    const current = invite;
    setInvite(null);
    setViewers([]);
    peersRef.current.forEach(peer => peer.close());
    peersRef.current.clear();
    streamRef.current?.getTracks().forEach(track => track.stop());
    streamRef.current = null;
    if(framebufferTimerRef.current){window.clearInterval(framebufferTimerRef.current);framebufferTimerRef.current=null;}
    socketRef.current?.close();
    socketRef.current = null;

    if (revokeOnServer && current) {
      try {
        await fetch(`${current.nodeUrl}/api/sessions/${encodeURIComponent(current.sessionId)}`, {
          method: "DELETE",
          headers: { authorization: `Bearer ${current.hostToken}` }
        });
      } catch {
        // Local revoke still closes all direct connections.
      }
    }
    setStatus("Remote session ended.");
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Remote Access</p><h3>Share this Android instance</h3></div>
        <span className={invite ? "pill" : "pill pill-warn"}>{invite ? "Session active" : "Offline"}</span>
      </div>

      <div className="remote-grid">
        <div className="tool-column">
          <label>Remote node URL
            <input value={settings.nodeUrl} disabled={Boolean(invite)} onChange={e => setSettings({...settings,nodeUrl:e.target.value})} />
          </label>
          <label>Fallback signalling node URLs
            <textarea value={settings.fallbackNodeUrls} disabled={Boolean(invite)} placeholder={"https://remote-eu.example.com\nhttps://remote-us.example.com"} onChange={e=>setSettings({...settings,fallbackNodeUrls:e.target.value})}/>
          </label>
          <label>Node secret
            <input type="password" value={settings.nodeSecret} disabled={Boolean(invite)} onChange={e => setSettings({...settings,nodeSecret:e.target.value})} />
          </label>
          <label>Android instance
            <select value={selected?.id ?? ""} disabled={Boolean(invite)} onChange={e => setSettings({...settings,instanceId:e.target.value})}>
              {instances.map(instance => <option value={instance.id} key={instance.id}>{instance.name}</option>)}
            </select>
          </label>
          <label>Invite lifetime
            <select value={settings.ttlSeconds} disabled={Boolean(invite)} onChange={e => setSettings({...settings,ttlSeconds:Number(e.target.value)})}>
              <option value={300}>5 minutes</option>
              <option value={900}>15 minutes</option>
              <option value={3600}>1 hour</option>
              <option value={14400}>4 hours</option>
              <option value={86400}>24 hours</option>
            </select>
          </label>

          <div className="permission-grid">
            <label><input type="checkbox" checked disabled /> View screen</label>
            <label><input type="checkbox" checked={settings.control} disabled={Boolean(invite)} onChange={e => setSettings({...settings,control:e.target.checked})} /> Control Android</label>
            <label><input type="checkbox" checked={settings.clipboard} disabled={Boolean(invite)} onChange={e => setSettings({...settings,clipboard:e.target.checked})} /> Clipboard (future)</label>
            <label><input type="checkbox" checked={settings.fileTransfer} disabled={Boolean(invite)} onChange={e => setSettings({...settings,fileTransfer:e.target.checked})} /> File transfer</label>
            <label><input type="checkbox" checked={settings.gamepad} disabled={Boolean(invite)} onChange={e => setSettings({...settings,gamepad:e.target.checked})} /> Gamepad forwarding</label>
            <label><input type="checkbox" checked={settings.unattendedTrusted} disabled={Boolean(invite)} onChange={e=>setSettings({...settings,unattendedTrusted:e.target.checked})} /> Allow trusted devices to reconnect without manual approval</label>
            <label><input type="checkbox" checked={settings.directFramebuffer} disabled={Boolean(invite)} onChange={e=>setSettings({...settings,directFramebuffer:e.target.checked})} /> Direct emulator framebuffer capture</label>
            <label><input type="checkbox" checked={settings.adaptiveBitrate} disabled={Boolean(invite)} onChange={e => setSettings({...settings,adaptiveBitrate:e.target.checked})} /> Adaptive bitrate</label>
          </div>

          <label>Remote FPS
            <select value={settings.fpsPreset} disabled={Boolean(invite)} onChange={e=>setSettings({...settings,fpsPreset:Number(e.target.value) as RemoteSettings["fpsPreset"]})}>
              {[30,60,90,120].map(v=><option key={v} value={v}>{v} FPS</option>)}
            </select>
          </label>

          {!invite ? (
            <button className="primary" disabled={busy || !selected} onClick={createInvite}>{busy ? "Creating…" : "Create Remote Invite"}</button>
          ) : (
            <button className="danger" onClick={() => endSession(true)}>End Remote Session</button>
          )}

          <div className="warning-box">
            Remote viewers require an invite. Manual approval remains the default; optional unattended access only auto-approves devices you explicitly trusted before.
          </div>
        </div>

        <div className="remote-session-card">
          {invite ? (
            <>
              <p className="eyebrow">Invite</p>
              <div className="invite-code">{invite.inviteCode}</div>
              <input readOnly value={invite.inviteUrl} onFocus={e => e.currentTarget.select()} />
              <small>Expires {new Date(invite.expiresAt).toLocaleString()}</small>

              <h4>Viewer requests</h4>
              {viewers.length === 0 ? <p className="muted">Nobody is waiting yet.</p> : viewers.map(viewer => (
                <div className="viewer-request" key={viewer.viewerId}>
                  <div>
                    <strong>{viewer.approved ? "Connected viewer" : "Approval requested"}{viewer.trusted ? " · trusted" : ""}</strong>
                    <small>{viewer.userAgent || viewer.viewerId}</small>
                  </div>
                  <div className="button-row">
                    {!viewer.approved ? (
                      <>
                        <button className="primary compact" onClick={() => approveViewer(viewer.viewerId)}>Approve</button>
                        <button className="ghost compact" onClick={() => {socketRef.current?.send(JSON.stringify({type:"viewer-trust",viewerId:viewer.viewerId}));setViewers(current=>current.map(v=>v.viewerId===viewer.viewerId?{...v,trusted:true}:v));}}>Trust device</button>
                        <button className="ghost compact" onClick={() => denyViewer(viewer.viewerId)}>Deny</button>
                      </>
                    ) : (
                      <button className="danger compact" onClick={() => revokeViewer(viewer.viewerId)}>Revoke</button>
                    )}
                  </div>
                </div>
              ))}
            </>
          ) : (
            <div className="empty compact-empty">
              <h4>Remote access is off</h4>
              <p>Create an expiring invite when you want somebody to view or control this Android instance from a browser or phone.</p>
            </div>
          )}
          <pre className="inline-output remote-status">{status}</pre>
        </div>
      </div>
    </section>
  );
}
