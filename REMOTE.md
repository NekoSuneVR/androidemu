# NekoDroid Remote Access

NekoDroid Remote is a self-hosted remote access layer for sharing one Android emulator instance with another PC, phone, tablet, or browser.

It is designed around explicit consent:

1. the host creates an expiring invite;
2. the remote user opens the invite URL or enters the invite code;
3. the host sees the viewer request;
4. the host approves or denies it;
5. screen sharing starts only after the host explicitly selects a screen/window;
6. Android control is enabled only if the host created the invite with control permission.

There is no hidden or unattended access path in the current design.

## Architecture

```text
Android Emulator / QEMU
        │
        │ ADB virtual input
        ▼
NekoDroid Desktop Host
        │
        │ WebRTC video/audio + control data channel
        │
        ├──────── direct P2P when possible ────────┐
        │                                           │
        ▼                                           ▼
Remote Node                                  Browser / Phone
(signalling + invites)                       / Another PC
        │
        └──────── TURN relay when required ─────────┘
```

The remote node does not need to carry video when a direct WebRTC connection works.

When NAT/firewall conditions prevent a direct connection, Coturn can relay the WebRTC traffic.

## Features currently implemented

- self-hosted remote node;
- Docker support;
- expiring invite codes;
- host authentication with a node secret;
- unique host session token;
- host approval/deny/revoke per viewer;
- view-only or view+control permissions;
- WebRTC signalling;
- STUN support;
- optional TURN relay;
- browser/mobile viewer;
- pointer/touch input;
- Android Back, Home and App Switch buttons;
- keyboard input;
- ADB-backed tap/swipe/key events;
- explicit screen/window capture prompt;
- multiple viewer request handling;
- session revocation.

Clipboard and remote file transfer permissions are reserved in the protocol/UI but are not marked complete yet.

## Deploy the remote node

```bash
cd remote-node
cp .env.example .env
```

Edit `.env`:

```env
PUBLIC_URL=https://remote.example.com
NODE_SECRET=use-a-long-random-secret
ALLOW_ORIGIN=*
PORT=8096

TURN_URL=turn:remote.example.com:3478
TURN_USERNAME=nekodroid
TURN_PASSWORD=use-another-long-random-password
TURN_REALM=remote.example.com
```

Start:

```bash
docker compose up -d --build
```

Health check:

```bash
curl https://remote.example.com/health
```

## Firewall

The signalling/web service needs the port you reverse-proxy to, normally:

```text
TCP 8096
```

In production it is better to put Nginx, Caddy, Traefik, or Nginx Proxy Manager in front of it and expose only HTTPS/WSS publicly.

Coturn requires:

```text
TCP/UDP 3478
UDP 49160-49200
```

If you change the Coturn relay port range in `docker-compose.yml`, update the firewall to match.

## TLS / HTTPS

For internet use, serve the remote website over HTTPS.

WebRTC browser behavior is more reliable in a secure context, and the node secret must not be sent over plain HTTP on an untrusted network.

Example proxy targets:

```text
https://remote.example.com
        ↓
http://127.0.0.1:8096
```

The reverse proxy must support WebSocket upgrades for:

```text
/ws
```

## Host configuration

Open:

```text
NekoDroid
  → Remote Access
```

Enter:

- remote node URL;
- node secret;
- Android instance;
- invite lifetime;
- whether the viewer may control the emulator.

Click **Create Remote Invite**.

NekoDroid shows:

- invite code;
- invite URL;
- expiry time;
- viewer requests.

When a viewer asks to connect, select:

- Approve;
- Deny;
- Revoke later.

On the first approved viewer, the host receives the normal operating-system screen/window sharing selector.

Choose the NekoDroid/QEMU Android display you want to expose.

## Browser / mobile connection

The viewer can open:

```text
https://remote.example.com/?invite=ABC123
```

or open the remote site and enter the invite code manually.

The viewer interface is touch friendly.

If control is allowed:

- tap -> Android tap;
- drag -> Android swipe;
- keyboard -> Android keyboard/key events;
- Back -> Android Back;
- Home -> Android Home;
- Apps -> Android App Switch.

The remote control path sends Android input through ADB instead of moving the host operating-system mouse.

## Connection strategy

Preferred path:

```text
Host ←──── encrypted WebRTC P2P ────→ Viewer
```

Fallback:

```text
Host ←──── WebRTC ────→ Coturn ←──── WebRTC ────→ Viewer
```

The signalling node coordinates the connection but does not need to proxy the video stream when direct P2P works.

## Cost

NekoDroid Remote itself is intended to be free and self-hostable.

There is no required paid remote-desktop service.

However:

- a VPS may have a hosting cost;
- TURN relay traffic consumes server bandwidth;
- a free-tier VPS may impose traffic or CPU limits.

If you already own a server/VPS or run the remote node at home, there is no additional software subscription fee.

## Security defaults

- invites expire;
- viewers cannot connect without an invite;
- viewers require host approval;
- session creation requires `NODE_SECRET`;
- each session receives a separate host token;
- sessions can be revoked immediately;
- control can be disabled while still allowing viewing;
- Android input is scoped to the selected emulator instance;
- LAN/public ADB exposure is not required;
- remote viewers do not receive the node secret.

## Future work

Planned remote features:

- direct framebuffer capture instead of host window capture;
- direct emulator audio capture;
- H.264/H.265/AV1 codec negotiation;
- adaptive bitrate;
- 30/60/90/120 FPS remote presets;
- clipboard synchronization;
- encrypted remote file transfer;
- controller/gamepad forwarding;
- remote microphone;
- remote virtual camera;
- persistent trusted-device pairing;
- optional unattended access with explicit owner configuration;
- account/organization mode for multiple NekoDroid hosts;
- multiple regional signalling/TURN nodes;
- connection quality statistics;
- Android native viewer app;
- iOS viewer app / PWA improvements.
