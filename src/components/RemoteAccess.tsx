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
};

type ViewerRequest = {
  viewerId: string;
  userAgent?: string;
  approved?: boolean;
};

type RemoteSettings = {
  nodeUrl: string;
  nodeSecret: string;
  instanceId: string;
  control: boolean;
  clipboard: boolean;
  fileTransfer: boolean;
  ttlSeconds: number;
};

const defaultSettings: RemoteSettings = {
  nodeUrl: "http://localhost:8096",
  nodeSecret: "",
  instanceId: "",
  control: true,
  clipboard: false,
  fileTransfer: false,
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
    peersRef.current.forEach(peer => peer.close());
  }, []);

  const createInvite = async () => {
    if (!selected) return setStatus("Create an Android instance first.");
    if (!settings.nodeUrl.trim() || !settings.nodeSecret) {
      return setStatus("Remote node URL and node secret are required.");
    }

    setBusy(true);
    try {
      const node = settings.nodeUrl.replace(/\/$/, "");
      const response = await fetch(`${node}/api/sessions`, {
        method: "POST",
        headers: {
          "content-type": "application/json",
          authorization: `Bearer ${settings.nodeSecret}`
        },
        body: JSON.stringify({
          name: selected.name,
          instanceId: selected.id,
          control: settings.control,
          clipboard: settings.clipboard,
          fileTransfer: settings.fileTransfer,
          ttlSeconds: settings.ttlSeconds
        })
      });

      const body = await response.json();
      if (!response.ok) throw new Error(body.error || `Remote node returned ${response.status}`);

      const created = body as Invite;
      setInvite(created);
      setViewers([]);
      setStatus("Invite created. Waiting for viewers.");
      connectHostSocket(node, created);
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
        : [...current, { viewerId: message.viewerId, userAgent: message.userAgent }]);
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

    if (message.type === "relay-data" && settings.control && message.payload) {
      await applyControl(message.viewerId, message.payload);
      return;
    }

    if (["session-expired", "session-revoked"].includes(message.type)) {
      await endSession(false);
    }
  };

  const ensureCapture = async () => {
    if (streamRef.current) return streamRef.current;
    if (!navigator.mediaDevices?.getDisplayMedia) {
      throw new Error("Screen capture is unavailable in this WebView. Use a current Tauri/WebView2 build.");
    }

    const stream = await navigator.mediaDevices.getDisplayMedia({
      video: { frameRate: 60 },
      audio: true
    });
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
      const configResponse = await fetch(`${settings.nodeUrl.replace(/\/$/, "")}/api/config`, {
        headers: { authorization: `Bearer ${settings.nodeSecret}` }
      });
      const config = configResponse.ok ? await configResponse.json() : { iceServers: [] };

      const peer = new RTCPeerConnection({ iceServers: config.iceServers || [] });
      peersRef.current.set(viewerId, peer);

      stream.getTracks().forEach(track => peer.addTrack(track, stream));

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
          await adbShell(`input tap ${x} ${y}`);
          return;
        }

        const distance = Math.hypot(x - start.x, y - start.y);
        if (distance < 12) {
          await adbShell(`input tap ${x} ${y}`);
        } else {
          const duration = Math.max(80, Math.min(1200, Date.now() - start.at));
          await adbShell(`input swipe ${start.x} ${start.y} ${x} ${y} ${duration}`);
        }
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
      if (key) await adbShell(`input keyevent ${key}`);
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
        await adbShell(`input keyevent ${keyMap[key]}`);
      } else if (key.length === 1 && /^[ -~]$/.test(key)) {
        const encoded = key === " " ? "%s" : key.replace(/[%&|<>]/g, "");
        if (encoded) await adbShell(`input text '${encoded}'`);
      }
    }
  };

  const adbShell = async (command: string) => {
    if (!selected) return;
    try {
      await invoke("adb_shell", { port: selected.adbPort, command });
    } catch (error) {
      setStatus(`Remote control failed: ${String(error)}`);
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
    socketRef.current?.close();
    socketRef.current = null;

    if (revokeOnServer && current) {
      try {
        await fetch(`${settings.nodeUrl.replace(/\/$/, "")}/api/sessions/${encodeURIComponent(current.sessionId)}`, {
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
            <label><input type="checkbox" checked={settings.fileTransfer} disabled={Boolean(invite)} onChange={e => setSettings({...settings,fileTransfer:e.target.checked})} /> File transfer (future)</label>
          </div>

          {!invite ? (
            <button className="primary" disabled={busy || !selected} onClick={createInvite}>{busy ? "Creating…" : "Create Remote Invite"}</button>
          ) : (
            <button className="danger" onClick={() => endSession(true)}>End Remote Session</button>
          )}

          <div className="warning-box">
            Remote viewers never gain access silently. Each viewer must use an unexpired invite and must be approved from this NekoDroid window.
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
                    <strong>{viewer.approved ? "Connected viewer" : "Approval requested"}</strong>
                    <small>{viewer.userAgent || viewer.viewerId}</small>
                  </div>
                  <div className="button-row">
                    {!viewer.approved ? (
                      <>
                        <button className="primary compact" onClick={() => approveViewer(viewer.viewerId)}>Approve</button>
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
