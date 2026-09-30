import http from "node:http";
import crypto from "node:crypto";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join, normalize } from "node:path";
import { WebSocketServer, WebSocket } from "ws";

const __dirname = dirname(fileURLToPath(import.meta.url));
const publicDir = join(__dirname, "..", "public");

const PORT = Number(process.env.PORT || 8096);
const PUBLIC_URL = process.env.PUBLIC_URL || `http://localhost:${PORT}`;
const DEFAULT_INVITE_TTL_SECONDS = Number(process.env.INVITE_TTL_SECONDS || 900);
const MAX_INVITE_TTL_SECONDS = Number(process.env.MAX_INVITE_TTL_SECONDS || 86400);
const NODE_SECRET = process.env.NODE_SECRET || "";
const TURN_URL = process.env.TURN_URL || "";
const TURN_USERNAME = process.env.TURN_USERNAME || "";
const TURN_PASSWORD = process.env.TURN_PASSWORD || "";
const ALLOW_ORIGIN = process.env.ALLOW_ORIGIN || "*";
const MAX_VIEWERS_PER_SESSION = Math.max(1, Math.min(Number(process.env.MAX_VIEWERS_PER_SESSION || 4), 32));

if (!NODE_SECRET) {
  console.warn("WARNING: NODE_SECRET is not set. Session creation is disabled until it is configured.");
}

const sessions = new Map();

function id(bytes = 18) {
  return crypto.randomBytes(bytes).toString("base64url");
}

function inviteCode() {
  return crypto.randomBytes(5).toString("hex").toUpperCase();
}

function send(ws, payload) {
  if (ws?.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(payload));
  }
}

function json(res, status, body) {
  const encoded = JSON.stringify(body);
  res.writeHead(status, {
    "content-type": "application/json; charset=utf-8",
    "cache-control": "no-store",
    "access-control-allow-origin": ALLOW_ORIGIN,
    "access-control-allow-headers": "authorization, content-type",
    "access-control-allow-methods": "GET,POST,DELETE,OPTIONS",
    "content-length": Buffer.byteLength(encoded)
  });
  res.end(encoded);
}

async function readJson(req) {
  let body = "";
  for await (const chunk of req) {
    body += chunk;
    if (body.length > 128 * 1024) throw new Error("Request too large");
  }
  return body ? JSON.parse(body) : {};
}

function cleanupExpired() {
  const now = Date.now();
  for (const [sessionId, session] of sessions) {
    if (session.expiresAt <= now) {
      send(session.hostSocket, { type: "session-expired" });
      for (const viewer of session.viewers.values()) {
        send(viewer.socket, { type: "session-expired" });
        viewer.socket?.close(4001, "Session expired");
      }
      session.hostSocket?.close(4001, "Session expired");
      sessions.delete(sessionId);
    }
  }
}

setInterval(cleanupExpired, 30_000).unref();

const server = http.createServer(async (req, res) => {
  try {
    const url = new URL(req.url, PUBLIC_URL);

    if (req.method === "OPTIONS") {
      res.writeHead(204, {
        "access-control-allow-origin": ALLOW_ORIGIN,
        "access-control-allow-headers": "authorization, content-type",
        "access-control-allow-methods": "GET,POST,DELETE,OPTIONS"
      });
      return res.end();
    }

    if (req.method === "GET" && url.pathname === "/health") {
      return json(res, 200, {
        ok: true,
        service: "nekodroid-remote-node",
        sessions: sessions.size,
        now: new Date().toISOString()
      });
    }

    if (req.method === "GET" && url.pathname === "/api/config") {
      const invite = String(url.searchParams.get("invite") || "").toUpperCase();
      const bearer = (req.headers.authorization || "").replace(/^Bearer\s+/i, "");
      const inviteValid = invite && [...sessions.values()].some(
        item => item.code === invite && item.expiresAt > Date.now()
      );
      const hostValid = NODE_SECRET && safeTextEqual(NODE_SECRET, bearer);

      if (!inviteValid && !hostValid) {
        return json(res, 401, { error: "Valid invite or host authentication required" });
      }

      const iceServers = [{ urls: "stun:stun.l.google.com:19302" }];
      if (TURN_URL) {
        iceServers.push({
          urls: TURN_URL,
          username: TURN_USERNAME,
          credential: TURN_PASSWORD
        });
      }
      return json(res, 200, { iceServers });
    }

    if (req.method === "POST" && url.pathname === "/api/sessions") {
      if (!NODE_SECRET) return json(res, 503, { error: "Remote node is not configured" });
      const bearer = (req.headers.authorization || "").replace(/^Bearer\s+/i, "");
      if (!safeTextEqual(NODE_SECRET, bearer)) {
        return json(res, 401, { error: "Invalid node secret" });
      }

      const body = await readJson(req);
      const ttlSeconds = Math.max(
        60,
        Math.min(Number(body.ttlSeconds || DEFAULT_INVITE_TTL_SECONDS), MAX_INVITE_TTL_SECONDS)
      );

      const sessionId = id(12);
      const hostToken = id(32);
      const code = inviteCode();
      const expiresAt = Date.now() + ttlSeconds * 1000;

      sessions.set(sessionId, {
        id: sessionId,
        code,
        name: String(body.name || "NekoDroid"),
        instanceId: String(body.instanceId || ""),
        permissions: {
          view: true,
          control: Boolean(body.control),
          clipboard: Boolean(body.clipboard),
          fileTransfer: Boolean(body.fileTransfer)
        },
        createdAt: Date.now(),
        expiresAt,
        hostTokenHash: sha256(hostToken),
        hostSocket: null,
        viewers: new Map()
      });

      return json(res, 201, {
        sessionId,
        hostToken,
        inviteCode: code,
        inviteUrl: `${PUBLIC_URL.replace(/\/$/, "")}/?invite=${encodeURIComponent(code)}`,
        expiresAt: new Date(expiresAt).toISOString()
      });
    }

    if (req.method === "GET" && url.pathname.startsWith("/api/invites/")) {
      cleanupExpired();
      const code = decodeURIComponent(url.pathname.slice("/api/invites/".length)).toUpperCase();
      const session = [...sessions.values()].find(item => item.code === code);

      if (!session) {
        return json(res, 404, { error: "Invite not found or expired" });
      }

      return json(res, 200, {
        sessionId: session.id,
        name: session.name,
        expiresAt: new Date(session.expiresAt).toISOString(),
        permissions: session.permissions
      });
    }

    if (req.method === "DELETE" && url.pathname.startsWith("/api/sessions/")) {
      const sessionId = decodeURIComponent(url.pathname.slice("/api/sessions/".length));
      const session = sessions.get(sessionId);
      if (!session) return json(res, 404, { error: "Session not found" });

      const bearer = (req.headers.authorization || "").replace(/^Bearer\s+/i, "");
      if (!safeHashEqual(session.hostTokenHash, sha256(bearer))) {
        return json(res, 401, { error: "Invalid host token" });
      }

      send(session.hostSocket, { type: "session-revoked" });
      for (const viewer of session.viewers.values()) {
        send(viewer.socket, { type: "session-revoked" });
        viewer.socket?.close(4002, "Session revoked");
      }
      session.hostSocket?.close(4002, "Session revoked");
      sessions.delete(sessionId);
      return json(res, 200, { ok: true });
    }

    if (req.method === "GET") {
      const relative = url.pathname === "/" ? "index.html" : url.pathname.slice(1);
      const safePath = normalize(relative).replace(/^(\.\.(\/|\\|$))+/, "");
      const filePath = join(publicDir, safePath);
      if (!filePath.startsWith(publicDir)) {
        res.writeHead(403); return res.end("Forbidden");
      }
      try {
        const bytes = await readFile(filePath);
        const ext = filePath.split(".").pop();
        const types = { html: "text/html; charset=utf-8", js: "text/javascript; charset=utf-8", css: "text/css; charset=utf-8" };
        res.writeHead(200, { "content-type": types[ext] || "application/octet-stream" });
        return res.end(bytes);
      } catch {
        res.writeHead(404); return res.end("Not found");
      }
    }

    res.writeHead(405);
    res.end("Method not allowed");
  } catch (error) {
    json(res, 400, { error: error.message || "Bad request" });
  }
});

const wss = new WebSocketServer({ server, path: "/ws" });

wss.on("connection", ws => {
  let identity = null;

  ws.on("message", raw => {
    try {
      const message = JSON.parse(String(raw));

      if (!identity) {
        if (message.type === "host-auth") {
          const session = sessions.get(String(message.sessionId || ""));
          if (!session || session.expiresAt <= Date.now()) {
            send(ws, { type: "error", error: "Session not found or expired" });
            return ws.close(4003, "Invalid session");
          }
          if (!safeHashEqual(session.hostTokenHash, sha256(String(message.hostToken || "")))) {
            send(ws, { type: "error", error: "Invalid host token" });
            return ws.close(4003, "Invalid token");
          }

          identity = { role: "host", sessionId: session.id };
          session.hostSocket = ws;
          send(ws, {
            type: "host-ready",
            sessionId: session.id,
            permissions: session.permissions,
            viewers: [...session.viewers.keys()]
          });
          return;
        }

        if (message.type === "viewer-auth") {
          const code = String(message.inviteCode || "").toUpperCase();
          const session = [...sessions.values()].find(item => item.code === code);
          if (!session || session.expiresAt <= Date.now()) {
            send(ws, { type: "error", error: "Invite not found or expired" });
            return ws.close(4003, "Invalid invite");
          }

          if (session.viewers.size >= MAX_VIEWERS_PER_SESSION) {
            send(ws, { type: "error", error: "This session has reached its viewer limit" });
            return ws.close(4006, "Viewer limit reached");
          }

          const viewerId = id(10);
          identity = { role: "viewer", sessionId: session.id, viewerId };
          session.viewers.set(viewerId, {
            socket: ws,
            approved: false,
            connectedAt: Date.now()
          });

          send(ws, {
            type: "viewer-waiting",
            viewerId,
            sessionId: session.id,
            name: session.name,
            permissions: session.permissions
          });

          send(session.hostSocket, {
            type: "viewer-request",
            viewerId,
            userAgent: String(message.userAgent || "").slice(0, 300)
          });
          return;
        }

        send(ws, { type: "error", error: "Authenticate first" });
        return;
      }

      const session = sessions.get(identity.sessionId);
      if (!session) return ws.close(4001, "Session unavailable");

      if (identity.role === "host") {
        handleHostMessage(session, message);
      } else {
        handleViewerMessage(session, identity.viewerId, message);
      }
    } catch {
      send(ws, { type: "error", error: "Invalid JSON message" });
    }
  });

  ws.on("close", () => {
    if (!identity) return;
    const session = sessions.get(identity.sessionId);
    if (!session) return;

    if (identity.role === "host") {
      if (session.hostSocket === ws) session.hostSocket = null;
      for (const viewer of session.viewers.values()) {
        send(viewer.socket, { type: "host-offline" });
      }
    } else {
      session.viewers.delete(identity.viewerId);
      send(session.hostSocket, { type: "viewer-left", viewerId: identity.viewerId });
    }
  });
});

function handleHostMessage(session, message) {
  const viewerId = String(message.viewerId || "");
  const viewer = session.viewers.get(viewerId);

  if (message.type === "viewer-approve" && viewer) {
    viewer.approved = true;
    send(viewer.socket, {
      type: "viewer-approved",
      viewerId,
      permissions: session.permissions
    });
    return;
  }

  if (message.type === "viewer-deny" && viewer) {
    send(viewer.socket, { type: "viewer-denied" });
    viewer.socket?.close(4004, "Denied by host");
    session.viewers.delete(viewerId);
    return;
  }

  if (message.type === "viewer-revoke" && viewer) {
    send(viewer.socket, { type: "viewer-revoked" });
    viewer.socket?.close(4005, "Access revoked by host");
    session.viewers.delete(viewerId);
    return;
  }

  if (["webrtc-offer", "webrtc-answer", "ice-candidate", "relay-data"].includes(message.type) && viewer?.approved) {
    send(viewer.socket, message);
  }
}

function handleViewerMessage(session, viewerId, message) {
  const viewer = session.viewers.get(viewerId);
  if (!viewer?.approved) {
    send(viewer?.socket, { type: "error", error: "Host approval required" });
    return;
  }

  if (["webrtc-offer", "webrtc-answer", "ice-candidate"].includes(message.type)) {
    send(session.hostSocket, { ...message, viewerId });
    return;
  }

  if (message.type === "relay-data") {
    if (!session.permissions.control) {
      send(viewer.socket, { type: "error", error: "Control permission is disabled" });
      return;
    }
    send(session.hostSocket, { ...message, viewerId });
  }
}

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest();
}

function safeHashEqual(a, b) {
  return Buffer.isBuffer(a) && Buffer.isBuffer(b) && a.length === b.length && crypto.timingSafeEqual(a, b);
}

function safeTextEqual(a, b) {
  const left = Buffer.from(String(a));
  const right = Buffer.from(String(b));
  return left.length === right.length && crypto.timingSafeEqual(left, right);
}

server.listen(PORT, "0.0.0.0", () => {
  console.log(`NekoDroid Remote Node listening on :${PORT}`);
  console.log(`Public URL: ${PUBLIC_URL}`);
});
