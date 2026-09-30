const joinCard = document.querySelector("#joinCard");
const viewer = document.querySelector("#viewer");
const joinForm = document.querySelector("#joinForm");
const inviteInput = document.querySelector("#invite");
const statusEl = document.querySelector("#status");
const waiting = document.querySelector("#waiting");
const permissionText = document.querySelector("#permissionText");
const sessionName = document.querySelector("#sessionName");
const video = document.querySelector("#screen");

let ws;
let pc;
let controlChannel;
let permissions = {};
let inviteCode = "";
let iceServers = [{ urls: "stun:stun.l.google.com:19302" }];

const params = new URLSearchParams(location.search);
if (params.get("invite")) inviteInput.value = params.get("invite");

joinForm.addEventListener("submit", async event => {
  event.preventDefault();
  inviteCode = inviteInput.value.trim().toUpperCase();
  if (!inviteCode) return;

  statusEl.textContent = "Checking invite…";
  const response = await fetch(`/api/invites/${encodeURIComponent(inviteCode)}`);
  if (!response.ok) {
    statusEl.textContent = "Invite not found or expired.";
    return;
  }

  const info = await response.json();
  const configResponse = await fetch(`/api/config?invite=${encodeURIComponent(inviteCode)}`);
  if (configResponse.ok) {
    const config = await configResponse.json();
    if (Array.isArray(config.iceServers)) iceServers = config.iceServers;
  }
  sessionName.textContent = info.name;
  permissions = info.permissions;

  ws = new WebSocket(`${location.protocol === "https:" ? "wss:" : "ws:"}//${location.host}/ws`);
  ws.addEventListener("open", () => {
    ws.send(JSON.stringify({
      type: "viewer-auth",
      inviteCode,
      userAgent: navigator.userAgent
    }));
  });
  ws.addEventListener("message", event => onMessage(JSON.parse(event.data)));
  ws.addEventListener("close", () => {
    waiting.textContent = "Remote session disconnected.";
    permissionText.textContent = "Disconnected";
  });

  joinCard.classList.add("hidden");
  viewer.classList.remove("hidden");
});

async function onMessage(message) {
  if (message.type === "viewer-waiting") {
    waiting.textContent = "Waiting for the host to approve this device…";
    permissionText.textContent = "Approval required";
    return;
  }

  if (message.type === "viewer-approved") {
    permissions = message.permissions;
    permissionText.textContent = permissions.control ? "View + control" : "View only";
    waiting.textContent = "Approved. Waiting for video…";
    createPeer();
    return;
  }

  if (message.type === "webrtc-offer") {
    createPeer();
    await pc.setRemoteDescription(message.sdp);
    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);
    ws.send(JSON.stringify({ type: "webrtc-answer", sdp: pc.localDescription }));
    return;
  }

  if (message.type === "ice-candidate" && message.candidate) {
    await pc?.addIceCandidate(message.candidate);
    return;
  }

  if (["viewer-denied","viewer-revoked","session-revoked","session-expired","host-offline"].includes(message.type)) {
    waiting.textContent = message.type.replaceAll("-", " ");
    permissionText.textContent = "Disconnected";
  }
}

function createPeer() {
  if (pc) return pc;
  pc = new RTCPeerConnection({ iceServers });
  pc.ontrack = event => {
    video.srcObject = event.streams[0];
    waiting.classList.add("hidden");
  };
  pc.onicecandidate = event => {
    if (event.candidate) ws.send(JSON.stringify({ type: "ice-candidate", candidate: event.candidate }));
  };
  pc.ondatachannel = event => {
    if (event.channel.label === "control") {
      controlChannel = event.channel;
    }
  };
  return pc;
}

function sendControl(payload) {
  if (!permissions.control) return;
  const encoded = JSON.stringify(payload);
  if (controlChannel?.readyState === "open") {
    controlChannel.send(encoded);
  } else if (ws?.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ type: "relay-data", payload }));
  }
}

function relativePoint(event) {
  const rect = video.getBoundingClientRect();
  return {
    x: Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width)),
    y: Math.max(0, Math.min(1, (event.clientY - rect.top) / rect.height))
  };
}

let pointerDown = false;
video.addEventListener("pointerdown", event => {
  if (!permissions.control) return;
  pointerDown = true;
  video.setPointerCapture(event.pointerId);
  sendControl({ kind: "pointer", action: "down", ...relativePoint(event) });
});
video.addEventListener("pointermove", event => {
  if (!pointerDown || !permissions.control) return;
  sendControl({ kind: "pointer", action: "move", ...relativePoint(event) });
});
video.addEventListener("pointerup", event => {
  if (!permissions.control) return;
  pointerDown = false;
  sendControl({ kind: "pointer", action: "up", ...relativePoint(event) });
});

document.addEventListener("keydown", event => {
  if (!permissions.control || ["INPUT","TEXTAREA"].includes(event.target.tagName)) return;
  sendControl({ kind: "key", action: "down", key: event.key, code: event.code });
  event.preventDefault();
});
document.addEventListener("keyup", event => {
  if (!permissions.control || ["INPUT","TEXTAREA"].includes(event.target.tagName)) return;
  sendControl({ kind: "key", action: "up", key: event.key, code: event.code });
  event.preventDefault();
});

document.querySelector("#clipboardSend")?.addEventListener("click", async () => {
  if (!permissions.clipboard) return;
  try {
    const text = await navigator.clipboard.readText();
    if (text) sendControl({ kind: "clipboard", text });
  } catch {
    const text = prompt("Clipboard text to send to Android");
    if (text) sendControl({ kind: "clipboard", text });
  }
});

let lastGamepadPacket = "";
setInterval(() => {
  if (!permissions.gamepad || !permissions.control) return;
  const pads = navigator.getGamepads?.() || [];
  const pad = [...pads].find(Boolean);
  if (!pad) return;
  const buttons = pad.buttons.map((b,i)=>b.pressed?i:null).filter(v=>v!==null);
  const packet = JSON.stringify({ kind:"gamepad", buttons, axes:pad.axes.map(v=>Math.round(v*1000)/1000) });
  if (packet !== lastGamepadPacket) {
    lastGamepadPacket = packet;
    sendControl(JSON.parse(packet));
  }
}, 100);

document.querySelectorAll("[data-key]").forEach(button => {
  button.addEventListener("click", () => sendControl({ kind: "android-key", key: button.dataset.key }));
});

document.querySelector("#disconnect").addEventListener("click", () => {
  controlChannel?.close();
  pc?.close();
  ws?.close();
  location.reload();
});

if ("serviceWorker" in navigator) {
  window.addEventListener("load", () => navigator.serviceWorker.register("/service-worker.js").catch(() => {}));
}
