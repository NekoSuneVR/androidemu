# NekoDroid

## Development Status

NekoDroid is now in active implementation.

The current development branch includes:

- Tauri 2 + React + TypeScript desktop launcher
- Rust backend command layer
- Persistent Android instance configs
- QEMU discovery and process lifecycle management
- KVM/WHPX accelerator capability probing
- Local Android image registry for x86_64 raw/QCOW2 disks
- Built-in phone, gaming phone, tablet, large-tablet and foldable profiles
- Per-instance CPU, RAM, Android version, profile, root mode and localhost ADB port
- ADB connect/disconnect/state
- ADB shell command execution
- APK installation
- ADB push/pull file transfer
- Runtime logs
- GitHub Actions build checks

See [BUILDING.md](BUILDING.md) for development prerequisites and current runtime limitations.

## NekoDroid Remote Access

The development branch now also contains a self-hosted remote-access system for sharing a NekoDroid Android instance with another PC, phone, tablet, or browser.

Current remote features include:

- self-hosted signalling/invite server;
- Docker deployment;
- WebRTC video/audio transport;
- optional Coturn relay for difficult NAT/firewall networks;
- expiring invite codes and URLs;
- explicit host approval, deny, and revoke;
- view-only or view+control permission;
- mobile-friendly browser viewer;
- Android touch/swipe/key forwarding through ADB;
- localhost-only emulator ADB remains private from the remote viewer.

See [REMOTE.md](REMOTE.md) for architecture and deployment instructions.

The remote software is self-hosted and has no required subscription. Cloud hosting or TURN bandwidth can still have infrastructure costs depending on where you deploy it.

> Android 16 x86_64 boot is not marked complete yet. The current QEMU layer boots a bootable disk image; native AOSP-style kernel/ramdisk/system/vendor/userdata boot support is the next runtime milestone.

---

NekoDroid is a planned high-performance Android gaming, development, automation, and media platform for Windows and Linux.

The goal is to provide a BlueStacks-style Android experience while remaining modular, developer-friendly, fast, and extensible.

NekoDroid is intended to support:

- Android 9 through Android 16+
- Phone, tablet, foldable, and custom device profiles
- High-performance hardware virtualization
- GPU-accelerated Android graphics
- Google Play-compatible Android images where legally and technically supported
- Custom AOSP/GSI images
- GrapheneOS-derived or GrapheneOS-compatible testing images where technically possible
- Multi-instance Android environments
- ADB and developer tools
- Optional root access
- PC-to-Android file transfer
- APK installation
- AI-assisted game control through virtual Android inputs
- Video/audio hardware acceleration and transcoding
- Virtual touchscreen, keyboard, mouse, gamepad, gyro, and sensor input
- Screen recording and streaming integration
- Portrait and landscape rotation
- Headless/off-screen Android operation
- Automation APIs

> NekoDroid should not attempt to falsify hardware-backed attestation, Play Integrity, anti-cheat measurements, or security identity. Device profiles are intended for compatibility, UI testing, gaming, and realistic Android form factors.

---

# Core Architecture

```text
NekoDroid
│
├── Desktop Launcher
│   ├── Home
│   ├── Game Library
│   ├── Google Play launcher
│   ├── APK Installer
│   ├── File Transfer
│   ├── Instance Manager
│   ├── Device Profiles
│   ├── Media Tools
│   ├── AI Assistant
│   └── Settings
│
├── Runtime Manager
│   ├── QEMU
│   ├── WHPX / Hyper-V acceleration
│   ├── KVM acceleration
│   ├── VirtIO
│   └── Snapshot Manager
│
├── Android Image Manager
│   ├── Android 9
│   ├── Android 10
│   ├── Android 11
│   ├── Android 12
│   ├── Android 12L
│   ├── Android 13
│   ├── Android 14
│   ├── Android 15
│   ├── Android 16
│   ├── Future Android releases
│   └── Custom GSI / AOSP images
│
├── Graphics
│   ├── Vulkan
│   ├── OpenGL ES
│   ├── ANGLE / gfxstream
│   ├── VirGL
│   ├── Shader Cache
│   └── Software fallback
│
├── Media Engine
│   ├── Android MediaCodec bridge
│   ├── FFmpeg
│   ├── NVENC / NVDEC
│   ├── Intel Quick Sync
│   ├── AMD AMF / VCN
│   └── CPU fallback
│
├── NekoAI
│   ├── Framebuffer Vision
│   ├── OCR
│   ├── Object Detection
│   ├── VLM / LLM Planner
│   ├── Game Skills
│   ├── Virtual Touch
│   ├── Virtual Gamepad
│   ├── Android Keyboard
│   └── Voice
│
├── Developer Tools
│   ├── ADB
│   ├── ADB Shell
│   ├── Logcat
│   ├── Package Manager
│   ├── Port Forwarding
│   ├── File Transfer
│   └── Debugging Tools
│
├── Privilege Manager
│   ├── Standard Mode
│   ├── Developer Mode
│   ├── ADB Root
│   └── Full Root
│
└── Virtual Input
    ├── Touch
    ├── Multi-touch
    ├── Keyboard
    ├── Mouse
    ├── Gamepad
    ├── Gyroscope
    ├── Accelerometer
    └── AI Input
```

---

# Android Versions

NekoDroid should use a modular image system.

```text
images/
├── android-9/
├── android-10/
├── android-11/
├── android-12/
├── android-12l/
├── android-13/
├── android-14/
├── android-15/
├── android-16/
└── custom/
```

Example image manifest:

```json
{
  "id": "android-16-x86_64",
  "name": "Android 16",
  "api": 36,
  "architecture": "x86_64",
  "type": "aosp",
  "system": "system.img",
  "vendor": "vendor.img",
  "userdata": "userdata.img",
  "recommended": true
}
```

Future Android releases should be installable by adding another image package rather than modifying the launcher.

---

# Android Device Profiles

NekoDroid should support realistic device form factors.

Available profiles may include:

- Phone
- Gaming Phone
- Tablet
- Large Tablet
- Foldable
- Custom Device

Example profile:

```text
Neko Gaming Phone

Resolution:       1080x2400
DPI:              420
Refresh Rate:     120 Hz
CPU:              8 virtual cores
RAM:              8 GB
Android:          16
Touch Points:     10
Gyroscope:        Enabled
Accelerometer:    Enabled
GPS:              Enabled
Bluetooth:        Enabled
Wi-Fi:            Enabled
```

A tablet profile may use:

```text
Resolution:       2560x1600
DPI:              280
Refresh Rate:     120 Hz
Telephony:        Disabled
Tablet Resources: Enabled
```

Profiles should change actual Android configuration instead of simply changing the profile name.

---

# Screen Rotation

NekoDroid should allow the Android display to switch between portrait and landscape.

Supported modes:

```text
Automatic
Portrait
Reverse Portrait
Landscape
Reverse Landscape
```

Example UI:

```text
Display Orientation

● Automatic
○ Portrait
○ Landscape
○ Reverse Portrait
○ Reverse Landscape

[ Rotate Left ] [ Rotate Right ]
```

Rotation should update:

- Android display orientation
- Touch coordinates
- AI vision coordinates
- Keyboard mappings
- Gamepad overlays
- Screen recording dimensions
- Streaming output
- Virtual camera output
- Window aspect ratio

The user should be able to rotate while Android is running.

---

# ADB

ADB should be optional and disabled by default.

Example:

```text
Developer Options

ADB                 ON
ADB over localhost  ON
ADB over network    OFF
Authentication      ON
Port                5555
```

Multiple instances may use:

```text
Gaming      127.0.0.1:5555
Testing     127.0.0.1:5556
Android 9   127.0.0.1:5557
```

NekoDroid should include:

- Connect/disconnect ADB
- ADB Shell
- Root ADB shell
- Logcat
- Package Manager
- `adb install`
- `adb uninstall`
- `adb push`
- `adb pull`
- Port forwarding
- Reverse port forwarding
- Reboot
- Reboot recovery
- Restart Android framework
- Screenshot
- Screen recording
- Device information
- Process viewer

---

# PC to Android File Transfer

Files should be transferable directly from the host PC into the Android instance.

Supported methods:

## Drag and Drop

```text
Windows file
    ↓
Drag onto NekoDroid
    ↓
/sdcard/Download/
```

Files may include:

- APK
- XAPK
- APKS
- ZIP
- MP3
- WAV
- FLAC
- MP4
- MKV
- WebM
- Images
- Documents
- Game data
- Custom files

## ADB Push

Example:

```bash
adb push myfile.zip /sdcard/Download/
```

The UI should provide the same functionality without requiring the terminal.

Example:

```text
Transfer File

Source:
C:\Users\User\Downloads\video.mp4

Destination:
/sdcard/Movies/

[ Transfer ]
```

## ADB Pull

Files should also be exportable from Android to the PC.

```text
Android:
/sdcard/DCIM/video.mp4

PC:
D:\AndroidExports\

[ Export ]
```

## APK Installation

APK drag-and-drop should automatically call the package installer.

```text
game.apk
   ↓
drop onto NekoDroid
   ↓
Install APK?
   ↓
Package installed
```

Optional advanced support:

- Split APK installation
- APKS bundles
- XAPK installation
- OBB/data extraction
- Install from URL
- Batch APK installation

---

# Root Access

Root should be configurable per Android instance.

Modes:

```text
Standard
Developer
ADB Root
Full Root
```

Example:

```text
Privilege Mode

● Standard
○ Developer
○ ADB Root
○ Full Root
```

Root mode may provide:

- `su`
- Root ADB shell
- Full filesystem access
- Writable system overlays
- Advanced debugging
- Root-only developer tools

A warning should explain that root may affect:

- Application compatibility
- Security
- Play Integrity
- DRM
- Games using anti-cheat
- Banking/security applications

Root should preferably be implemented using separate snapshots or boot configurations.

---

# Google Play

Google Play support should be modular.

Possible image types:

```text
AOSP
Google Play-compatible image
Custom GApps image
microG image
Custom GSI
GrapheneOS-derived/testing image
```

Google Mobile Services are proprietary, so distribution must follow Google's licensing and certification requirements.

The system should not claim hardware-backed security properties that the virtual device does not possess.

---

# GrapheneOS

GrapheneOS support should be treated as an experimental/custom image feature.

NekoDroid should not pretend that a VM has:

- Pixel hardware security
- Hardware-backed attestation
- Titan security hardware
- Genuine Pixel device identity

A GrapheneOS-derived testing image may still be useful for:

- Android application testing
- UI testing
- Privacy testing
- Development
- Compatibility experiments

---

# Performance Mode

Default gaming mode:

```text
Android x86_64
     ↓
Hardware Virtualization
     ↓
ARM Translation when required
     ↓
GPU Acceleration
```

Avoid pure software CPU emulation whenever possible.

Host acceleration:

```text
Windows:
WHPX / Hyper-V-compatible backend

Linux:
KVM
```

---

# ARM Compatibility

Android applications may contain:

```text
x86
x86_64
armeabi-v7a
arm64-v8a
```

NekoDroid should inspect installed packages and determine the best execution path.

```text
x86_64 available?
    YES → native execution
    NO
     ↓
ARM64 available?
    YES → ARM translation / compatible runtime
```

Possible native bridge support:

- `libndk_translation`
- Other legally redistributable compatible translation technologies
- ARM64 guest mode

---

# Graphics

Preferred graphics path:

```text
Android Game
    ↓
OpenGL ES / Vulkan
    ↓
gfxstream / ANGLE / VirGL
    ↓
Host Vulkan / DirectX / OpenGL
    ↓
GPU
```

Supported modes:

```text
Automatic
Vulkan
DirectX bridge
OpenGL
Software
```

Performance options:

- Shader cache
- ASTC cache
- Frame pacing
- VSync
- Triple buffering
- FPS limiter
- High FPS mode
- Resolution scaling
- Dynamic resolution
- GPU memory limits

---

# FPS

Selectable frame rates:

```text
30
60
90
120
144
165
240
Unlimited
```

The runtime should not force applications to exceed their own supported refresh rate.

---

# Virtual Input

One unified Android input API should control all virtual input.

```text
AndroidInput.tap()
AndroidInput.hold()
AndroidInput.swipe()
AndroidInput.multiTouch()
AndroidInput.key()
AndroidInput.text()
AndroidInput.gamepad.button()
AndroidInput.gamepad.axis()
AndroidInput.gyro()
AndroidInput.accelerometer()
```

Input sources:

```text
Keyboard Mapper ─┐
Mouse Mapper ────┤
Controller ──────┼──> Virtual Android Input
AI ──────────────┤
Automation API ──┘
```

---

# Keymapping

Example:

```text
WASD       → Virtual joystick
Mouse      → Camera
Left Click → Attack
Space      → Jump
E          → Interact
Shift      → Sprint
```

Example configuration:

```json
{
  "package": "com.example.game",
  "controls": [
    {
      "type": "joystick",
      "x": 250,
      "y": 800,
      "keys": {
        "up": "W",
        "down": "S",
        "left": "A",
        "right": "D"
      }
    },
    {
      "type": "tap",
      "key": "SPACE",
      "x": 1700,
      "y": 850
    }
  ]
}
```

---

# NekoAI

NekoAI is an optional free/local AI gaming assistant.

It should control Android through the virtual Android input system rather than moving the host Windows cursor.

```text
Android Framebuffer
       ↓
Vision
       ↓
OCR / Object Detection
       ↓
Game State
       ↓
Planner
       ↓
Action Validator
       ↓
Virtual Android Input
       ↓
Game
```

Possible AI components:

- YOLO
- OpenCV
- OCR
- Qwen
- Gemma
- Llama
- Qwen-VL style vision models
- Ollama
- whisper.cpp
- Piper
- Edge TTS/system voices

AI providers:

```text
Local Ollama
Remote Ollama
OpenAI-compatible endpoint
Built-in lightweight models
```

---

# AI Game Skills

Games may have dedicated skill folders.

```text
skills/
├── generic-android/
├── minecraft/
├── game-example/
└── custom/
```

Example skill:

```json
{
  "package": "com.example.game",
  "orientation": "landscape",
  "controls": {
    "move": "virtual-stick-left",
    "camera": "drag-right",
    "jump": [1750, 780]
  }
}
```

AI modes:

```text
Assistant Mode
Full Automation Mode
Accessibility Mode
Manual Mode
```

For fast gameplay, lightweight control logic should handle high-frequency movement while an LLM handles higher-level decisions.

---

# Framebuffer Access

AI vision should avoid repeatedly using:

```bash
adb exec-out screencap
```

for high-frequency analysis.

Preferred architecture:

```text
Android GPU framebuffer
        ↓
Shared Memory
        ├── Display
        ├── AI Vision
        ├── Screenshot
        ├── Recording
        └── Streaming
```

Possible configuration:

```text
Game rendering: 120 FPS
AI vision:       20-30 FPS
```

---

# Media Engine

NekoDroid should provide strong audio/video codec support.

Android side:

- MediaCodec
- MediaExtractor
- MediaMuxer
- Codec2-compatible interfaces

Host side:

- FFmpeg
- NVIDIA NVENC/NVDEC
- Intel Quick Sync
- AMD AMF/VCN
- Software fallback

Pipeline:

```text
Android App
    ↓
MediaCodec
    ↓
NekoDroid Codec Bridge
    ↓
Host GPU / CPU
```

---

# Video Transcoding

Built-in media tools may include:

- Convert video
- Convert audio
- Compress video
- Extract audio
- Remux
- Change resolution
- Change FPS
- Hardware encode
- Hardware decode
- Batch conversion

Example:

```text
Input:
video.mkv

Decode:
HEVC

Scale:
1920x1080

Encode:
H.264 NVENC

Output:
video.mp4
```

---

# Codec Support

Desired codec support:

## Video

- H.264 / AVC
- H.265 / HEVC
- VP8
- VP9
- AV1
- MPEG-4
- MPEG-2 where required

## Audio

- AAC
- MP3
- Opus
- Vorbis
- FLAC
- WAV / PCM
- ALAC where available

The runtime should query actual host hardware capabilities and use software fallbacks where required.

---

# Streaming and Recording

One framebuffer source should feed:

```text
Display
AI
Screenshots
Recording
OBS output
Virtual Camera
Streaming
```

Potential future outputs:

- OBS-compatible capture
- Virtual webcam
- NDI-style output
- Spout-style output
- RTMP
- SRT
- WebRTC

---

# Multi-Instance Manager

Each Android instance should have separate:

- Userdata
- Android version
- Device profile
- CPU
- RAM
- GPU settings
- ADB port
- Root mode
- AI settings
- Keybindings
- Filesystem
- Snapshots

Example:

```text
Gaming
Android 16
8 GB RAM
120 FPS
ADB: 5555

Legacy Apps
Android 9
4 GB RAM
60 FPS
ADB: 5556

Testing
Android 15
Tablet
6 GB RAM
ADB: 5557
```

Storage example:

```text
instances/
├── gaming/
│   ├── userdata.qcow2
│   ├── config.json
│   └── snapshots/
├── android9/
│   ├── userdata.qcow2
│   └── config.json
└── testing/
    ├── userdata.qcow2
    └── config.json
```

---

# Snapshots

Support:

- Create snapshot
- Restore snapshot
- Clone instance
- Clean boot
- Factory reset
- Root snapshot
- Non-root snapshot

Snapshots should use copy-on-write storage where possible.

---

# File Manager

Built-in Android file manager:

```text
/
├── data
├── sdcard
├── system
├── vendor
└── storage
```

Normal mode should primarily expose user-accessible storage.

Root mode may expose the full filesystem.

Features:

- Upload
- Download
- Drag/drop
- Delete
- Rename
- Copy
- Move
- Create folder
- Permissions viewer
- File properties

---

# Developer Terminal

Built-in terminal modes:

```text
ADB Shell
Root Shell
Logcat
Kernel Log
Package Manager
Processes
Network
Storage
```

Useful commands:

```bash
pm list packages
dumpsys activity
dumpsys SurfaceFlinger
getprop
top
df
settings
am
cmd
```

---

# Automation API

Example REST interface:

```text
GET  /instances
POST /instances/:id/start
POST /instances/:id/stop

GET  /instances/:id/frame

POST /instances/:id/input/tap
POST /instances/:id/input/swipe
POST /instances/:id/input/text
POST /instances/:id/input/gamepad

POST /instances/:id/apk/install

POST /instances/:id/files/push
POST /instances/:id/files/pull

POST /instances/:id/adb

POST /instances/:id/rotate
POST /instances/:id/reboot
```

WebSocket endpoints may include:

```text
/ws/frame
/ws/audio
/ws/logcat
/ws/input
/ws/ai
```

---

# Security

Recommended defaults:

```text
ADB                     OFF
ADB Network Exposure    OFF
Root                    OFF
AI Control              OFF
Automation API          localhost only
```

ADB network mode should require explicit opt-in.

Root should display a security warning.

Automation and AI inputs should be visible and stoppable by the user.

---

# Suggested Technology Stack

Desktop:

```text
Tauri
React
TypeScript
Rust
```

Virtualization:

```text
QEMU
WHPX / Hyper-V acceleration
KVM
VirtIO
QCOW2
```

Android:

```text
AOSP
x86_64
ARM translation/native bridge
Custom GSI support
```

Graphics:

```text
Vulkan
gfxstream
ANGLE
VirGL
```

Media:

```text
FFmpeg
NVENC/NVDEC
Intel Quick Sync
AMD AMF
```

AI:

```text
Ollama
ONNX Runtime
YOLO
OpenCV
OCR
whisper.cpp
Piper / Edge TTS
```

---

# Project Goals

NekoDroid should aim to be:

- Fast
- Modular
- Free to use
- Friendly to developers
- Useful for gaming
- Useful for Android testing
- Useful for AI automation
- Useful for media applications
- Easy to debug
- Easy to extend
- Free from unnecessary advertising services
- Transparent about virtualization and device-security limitations

