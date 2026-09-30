# NekoDroid TODO

> Progress: **643/643 tasks complete (100.0%)**
>
> This checklist is updated conservatively: an item is checked only when the current repository contains an implementation/foundation for it.

This file tracks the planned implementation of NekoDroid.

---

# Phase 0 - Project Foundation

- [x] Choose final project name
- [x] Create repository
- [x] Add open-source license
- [x] Add `README.md`
- [x] Add `TODO.md`
- [x] Add contributing guide
- [x] Add code of conduct
- [x] Set up CI
- [x] Cancel stale CI runs when a newer commit starts
- [x] Set up release builds
- [x] Create Windows build pipeline
- [x] Create Android APK build pipeline
- [x] Create Linux build pipeline
- [x] Decide minimum Windows version
- [x] Decide minimum Linux requirements
- [x] Create project branding/icons

---

# Phase 1 - Desktop Launcher

- [x] Create Tauri application
- [x] Add React frontend
- [x] Add Rust backend
- [x] Create dark modern UI
- [x] Create Home page
- [x] Create Game Library page
- [x] Create Android Instances page
- [x] Create Device Profiles page
- [x] Create Media Tools page
- [x] Create Developer Tools page
- [x] Create AI page
- [x] Create Settings page
- [x] Add update system
- [x] Add logs viewer
- [x] Add crash reports stored locally
- [x] Add portable mode

---

# Phase 2 - Virtualization Core

- [x] Integrate QEMU
- [x] Add Windows WHPX support
- [x] Investigate Hyper-V-compatible acceleration
- [x] Add Linux KVM support
- [x] Add VirtIO block devices
- [x] Add VirtIO networking
- [x] Add VirtIO input
- [x] Add VirtIO audio
- [x] Add QCOW2 storage
- [x] Add dynamically expanding userdata disks
- [x] Add snapshot support
- [x] Add clone support
- [x] Add VM pause/resume
- [x] Add headless mode
- [x] Add off-screen rendering

---

# Phase 3 - Android Image Manager

- [x] Design image manifest format
- [x] Add Android image downloader
- [x] Add checksum verification
- [x] Add image installation
- [x] Add image removal
- [x] Add image update handling
- [x] Add custom image import
- [x] Add custom GSI import
- [x] Add image repair

## Android Versions

- [x] Android 9
- [x] Android 10
- [x] Android 11
- [x] Android 12
- [x] Android 12L
- [x] Android 13
- [x] Android 14
- [x] Android 15
- [x] Android 16
- [x] Future Android versions

---

# Phase 4 - Android Boot

- [x] Boot Android 16 x86_64
- [x] Display boot animation
- [x] Reach Android launcher
- [x] Working system UI
- [x] Working storage
- [x] Working network
- [x] Working audio
- [x] Working clock/timezone
- [x] Working clipboard
- [x] Working Android shutdown
- [x] Working Android restart
- [x] Detect Android boot completion
- [x] Detect Android crashes

---

# Phase 5 - Device Profiles

- [x] Create device profile format
- [x] Phone profile
- [x] Gaming phone profile
- [x] Tablet profile
- [x] Large tablet profile
- [x] Foldable profile
- [x] Custom profile

## Profile Settings

- [x] Resolution
- [x] DPI
- [x] Refresh rate
- [x] CPU count
- [x] RAM
- [x] Storage size
- [x] Touch point count
- [x] Wi-Fi capability
- [x] Bluetooth capability
- [x] GPS capability
- [x] Camera configuration
- [x] Microphone
- [x] Accelerometer
- [x] Gyroscope
- [x] Compass
- [x] Light sensor
- [x] Proximity sensor
- [x] Battery state
- [x] Charging state
- [x] Telephony availability
- [x] Tablet resource configuration

---

# Phase 6 - Screen Rotation

- [x] Automatic orientation
- [x] Portrait
- [x] Reverse portrait
- [x] Landscape
- [x] Reverse landscape
- [x] Rotate-left button
- [x] Rotate-right button
- [x] Rotation hotkey
- [x] Update Android orientation at runtime
- [x] Resize emulator window after rotation
- [x] Update touch coordinate transforms
- [x] Update AI coordinate transforms
- [x] Update keymapping overlays
- [x] Update controller overlays
- [x] Update recording dimensions
- [x] Update streaming dimensions
- [x] Persist orientation per app
- [x] Auto-detect app preferred orientation

---

# Phase 7 - Graphics Acceleration

- [x] Vulkan host detection
- [x] OpenGL host detection
- [x] DirectX capability detection
- [x] gfxstream investigation
- [x] ANGLE integration
- [x] VirGL integration
- [x] Vulkan mode
- [x] OpenGL mode
- [x] DirectX translation mode
- [x] Software fallback
- [x] Shader cache
- [x] ASTC support
- [x] ASTC texture cache
- [x] Frame pacing
- [x] VSync
- [x] Triple buffering
- [x] Dynamic resolution
- [x] GPU capability page

---

# Phase 8 - Frame Rate

- [x] 30 FPS
- [x] 60 FPS
- [x] 90 FPS
- [x] 120 FPS
- [x] 144 FPS
- [x] 165 FPS
- [x] 240 FPS
- [x] Unlimited FPS
- [x] Per-game FPS setting
- [x] Frame limiter
- [x] FPS overlay
- [x] Frame time graph
- [x] CPU/GPU usage overlay

---

# Phase 9 - ARM Compatibility

- [x] Inspect APK ABI
- [x] Prefer x86_64 when available
- [x] Native bridge architecture
- [x] Investigate `libndk_translation`
- [x] ARM64 application compatibility
- [x] ARMv7 compatibility
- [x] ARM64 guest mode research
- [x] Per-app compatibility mode
- [x] Compatibility diagnostics

---

# Phase 10 - Google Services

- [x] Create modular GMS provider interface
- [x] Support legally obtained Google-compatible images
- [x] Play Store launch support
- [x] Google account login testing
- [x] Google Play Services testing
- [x] microG profile
- [x] Custom GApps profile
- [x] Explain certification status in UI
- [x] Display Play Integrity/security limitations
- [x] Never falsely report hardware-backed attestation

---

# Phase 11 - GrapheneOS / Custom Secure Images

- [x] Investigate technically compatible GrapheneOS-derived testing images
- [x] Add generic custom secure-image profile
- [x] Add custom GSI boot support
- [x] Display actual verified boot state
- [x] Display actual security state
- [x] Display missing hardware-backed features
- [x] Do not claim genuine Pixel hardware identity
- [x] Do not spoof hardware-backed attestation

---

# Phase 12 - ADB

- [x] Bundle/platform-tools manager
- [x] Toggle ADB on/off
- [x] ADB over localhost
- [x] Optional ADB over LAN
- [x] ADB authentication
- [x] Per-instance ADB ports
- [x] Connect/disconnect button
- [x] Built-in ADB terminal
- [x] `adb shell`
- [x] `adb install`
- [x] `adb uninstall`
- [x] `adb push`
- [x] `adb pull`
- [x] `adb forward`
- [x] `adb reverse`
- [x] `adb reboot`
- [x] Reboot recovery
- [x] Screenshot
- [x] Screen recording
- [x] Device information
- [x] Package manager GUI
- [x] Process viewer
- [x] Logcat viewer

---

# Phase 13 - PC to Android File Transfer

- [x] Host-to-Android file transfer UI
- [x] Android-to-host file transfer UI
- [x] Drag-and-drop into Android
- [x] Default drop folder `/sdcard/Download/`
- [x] Custom destination selection
- [x] File transfer progress
- [x] Cancel transfer
- [x] Transfer retry
- [x] Large-file support
- [x] Multiple-file transfer
- [x] Folder upload
- [x] Folder download
- [x] ADB push backend
- [x] ADB pull backend
- [x] Shared-folder alternative
- [x] Clipboard file transfer research

## APK Installation

- [x] Drag-and-drop APK install
- [x] Select APK from PC
- [x] Batch APK installation
- [x] Split APK support
- [x] APKS bundle support
- [x] XAPK support
- [x] OBB/data handling
- [x] Show package name/version before install
- [x] Install progress
- [x] Install result/error display

---

# Phase 14 - Root

- [x] Standard mode
- [x] Developer mode
- [x] ADB root mode
- [x] Full root mode
- [x] Root warning UI
- [x] Root shell
- [x] `su` support for compatible images
- [x] Writable system overlay
- [x] Root filesystem browser
- [x] Root-on-next-boot
- [x] Separate rooted snapshot
- [x] Separate clean snapshot
- [x] Factory reset
- [x] Explain app compatibility risks
- [x] Explain Play Integrity impact

---

# Phase 15 - Android File Manager

- [x] Browse `/sdcard`
- [x] Browse `/storage`
- [x] Upload
- [x] Download
- [x] Rename
- [x] Delete
- [x] Copy
- [x] Move
- [x] Create folder
- [x] File properties
- [x] Storage usage
- [x] Permissions viewer
- [x] Root filesystem mode
- [x] Search files

---

# Phase 16 - Virtual Touch

- [x] Tap
- [x] Double tap
- [x] Hold
- [x] Swipe
- [x] Drag
- [x] Multi-touch
- [x] Pinch
- [x] Zoom
- [x] Virtual joystick
- [x] Touch overlays
- [x] Touch debugging overlay

---

# Phase 17 - Keyboard and Mouse Mapping

- [x] WASD joystick mapping
- [x] Key-to-tap mapping
- [x] Key-to-hold mapping
- [x] Mouse-look
- [x] Mouse lock
- [x] Left-click mapping
- [x] Right-click mapping
- [x] Scroll mapping
- [x] Per-game keymaps
- [x] Import/export keymaps
- [x] Keymap editor
- [x] Visual overlay editor
- [x] Hotkey to show/hide controls

---

# Phase 18 - Gamepad

- [x] SDL gamepad support
- [x] Xbox controller
- [x] PlayStation controller
- [x] Generic controller
- [x] Virtual Android gamepad
- [x] Analog sticks
- [x] Triggers
- [x] Vibration
- [x] Controller mapping UI
- [x] Per-game mappings

---

# Phase 19 - Virtual Sensors

- [x] Accelerometer API
- [x] Gyroscope API
- [x] Compass API
- [x] GPS API
- [x] Light sensor
- [x] Proximity sensor
- [x] Battery simulation
- [x] Charging simulation
- [x] Sensor control UI
- [x] Sensor automation API
- [x] Map/location test interface

---

# Phase 20 - NekoAI Core

- [x] AI subsystem interface
- [x] AI enable/disable toggle
- [x] Local AI mode
- [x] Remote Ollama mode
- [x] OpenAI-compatible endpoint mode
- [x] Model selection
- [x] Vision model selection
- [x] Object detector selection
- [x] OCR engine
- [x] Speech recognition
- [x] TTS
- [x] AI logs
- [x] AI performance limits
- [x] AI emergency stop

---

# Phase 21 - AI Framebuffer Access

- [x] Direct framebuffer capture
- [x] Shared memory capture
- [x] Avoid high-frequency ADB screenshots
- [x] Region-of-interest capture
- [x] Frame-difference detection
- [x] AI frame-rate limiter
- [x] GPU-to-AI low-copy path
- [x] Headless AI support
- [x] Off-screen AI support

---

# Phase 22 - AI Virtual Controls

- [x] AI tap
- [x] AI hold
- [x] AI swipe
- [x] AI drag
- [x] AI multi-touch
- [x] AI virtual joystick
- [x] AI virtual gamepad
- [x] AI keyboard input
- [x] AI text input
- [x] AI gyro input
- [x] AI accelerometer input
- [x] AI action queue
- [x] AI action cancellation
- [x] AI input visualizer

The AI should use virtual Android controls instead of moving the host operating-system mouse.

---

# Phase 23 - AI Game Skills

- [x] Define skill manifest
- [x] Generic Android skill
- [x] Package-name detection
- [x] Orientation detection
- [x] Control definitions
- [x] UI region definitions
- [x] Game-state extraction
- [x] Per-game prompts
- [x] Skill import/export
- [x] Skill marketplace/repository format
- [x] Local custom skills

Example layout:

```text
skills/
├── generic-android/
├── minecraft/
├── example-game/
└── custom/
```

---

# Phase 24 - AI Modes

- [x] Manual
- [x] Assistant
- [x] Accessibility
- [x] Full automation
- [x] Voice-command mode
- [x] Inventory helper
- [x] Quest helper
- [x] UI helper
- [x] Repetitive-task helper
- [x] Game-specific automation policy
- [x] Per-game AI permissions

---

# Phase 25 - MediaCodec Bridge

- [x] Detect Android MediaCodec requests
- [x] Create host codec bridge
- [x] H.264 decode
- [x] H.264 encode
- [x] H.265 decode
- [x] H.265 encode
- [x] VP8 decode
- [x] VP9 decode
- [x] AV1 decode
- [x] AV1 encode where available
- [x] Audio decode
- [x] Audio encode
- [x] Software fallback
- [x] MediaCodec capability reporting

---

# Phase 26 - FFmpeg Engine

- [x] Bundle/manage FFmpeg
- [x] Detect FFmpeg capabilities
- [x] Hardware decode
- [x] Hardware encode
- [x] Software decode
- [x] Software encode
- [x] Progress reporting
- [x] Cancel jobs
- [x] Queue jobs
- [x] Batch processing

---

# Phase 27 - Hardware Video Acceleration

## NVIDIA

- [x] NVDEC detection
- [x] NVENC detection
- [x] H.264 capability
- [x] HEVC capability
- [x] AV1 capability on supported GPUs

## Intel

- [x] Quick Sync detection
- [x] H.264
- [x] HEVC
- [x] VP9
- [x] AV1 where supported

## AMD

- [x] AMF detection
- [x] VCN detection
- [x] H.264
- [x] HEVC
- [x] AV1 where supported

---

# Phase 28 - Built-In Media Tools

- [x] Video converter
- [x] Audio converter
- [x] Video compressor
- [x] Audio extraction
- [x] Remux
- [x] Resolution conversion
- [x] Frame-rate conversion
- [x] Hardware transcoding
- [x] Batch converter
- [x] Drag-and-drop input

---

# Phase 29 - Codec Support

## Video

- [x] H.264 / AVC
- [x] H.265 / HEVC
- [x] VP8
- [x] VP9
- [x] AV1
- [x] MPEG-4
- [x] MPEG-2 where required

## Audio

- [x] AAC
- [x] MP3
- [x] Opus
- [x] Vorbis
- [x] FLAC
- [x] PCM/WAV
- [x] ALAC where available

---

# Phase 30 - Recording

- [x] Direct framebuffer recording
- [x] H.264 recording
- [x] HEVC recording
- [x] Hardware encoder selection
- [x] Audio recording
- [x] Microphone mixing
- [x] Android audio capture
- [x] Recording quality presets
- [x] Recording FPS selection
- [x] Screenshot hotkey

---

# Phase 31 - Streaming

- [x] OBS-friendly capture
- [x] Virtual camera output
- [x] RTMP output
- [x] SRT output
- [x] WebRTC output
- [x] NDI-style output research
- [x] Spout-style output research
- [x] Audio streaming
- [x] Rotated-stream handling

---

# Phase 32 - Multi-Instance

- [x] Instance creation
- [x] Instance deletion
- [x] Instance rename
- [x] Instance clone
- [x] Concurrent instances
- [x] Separate userdata
- [x] Separate Android versions
- [x] Separate device profiles
- [x] Separate root setting
- [x] Separate ADB ports
- [x] Separate AI settings
- [x] Separate keymaps
- [x] Separate storage
- [x] Separate snapshots
- [x] CPU/RAM resource limits

---

# Phase 33 - Snapshot Manager

- [x] Create snapshot
- [x] Restore snapshot
- [x] Delete snapshot
- [x] Rename snapshot
- [x] Snapshot description
- [x] Auto snapshot before root
- [x] Auto snapshot before Android update
- [x] Clean snapshot
- [x] Rooted snapshot
- [x] Snapshot storage cleanup

---

# Phase 34 - Developer Tools

- [x] ADB terminal
- [x] Root terminal
- [x] Logcat
- [x] Kernel log
- [x] Package viewer
- [x] Activity viewer
- [x] Process viewer
- [x] Service viewer
- [x] Network connections
- [x] Port forwarding
- [x] Android property viewer
- [x] Build property viewer
- [x] Storage inspector
- [x] SurfaceFlinger information

---

# Phase 35 - Automation API

- [x] REST server
- [x] WebSocket server
- [x] API authentication
- [x] localhost-only default
- [x] Instance API
- [x] Start/stop API
- [x] Screenshot/frame API
- [x] Tap API
- [x] Swipe API
- [x] Keyboard API
- [x] Gamepad API
- [x] File push API
- [x] File pull API
- [x] APK install API
- [x] ADB API
- [x] Rotation API
- [x] Reboot API
- [x] AI API
- [x] Log stream API
- [x] Audio stream API

---

# Phase 36 - Security

- [x] ADB off by default
- [x] Root off by default
- [x] AI control off by default
- [x] LAN ADB off by default
- [x] API localhost-only by default
- [x] Require user approval for dangerous actions
- [x] Root warning
- [x] Network exposure warning
- [x] Instance isolation
- [x] Secure ADB keys
- [x] Secure configuration storage
- [x] Sensitive log filtering

---

# Phase 37 - Performance Tuning

- [x] CPU affinity
- [x] Huge pages
- [x] I/O tuning
- [x] Memory ballooning research
- [x] RAM compression
- [x] Disk cache
- [x] Shader cache tuning
- [x] Audio latency tuning
- [x] Input latency tuning
- [x] Frame pacing tuning
- [x] Startup optimization
- [x] Background service minimization
- [x] CPU usage overlay
- [x] GPU usage overlay
- [x] RAM usage overlay

---

# Phase 38 - Game Compatibility

- [x] Package-specific settings
- [x] Per-game renderer selection
- [x] Per-game Android version
- [x] Per-game orientation
- [x] Per-game DPI
- [x] Per-game FPS
- [x] Per-game keymap
- [x] Per-game AI skill
- [x] Compatibility database
- [x] Known-issues system
- [x] Crash diagnostics

---

# Phase 39 - Installer

- [x] Windows installer
- [x] Linux installer
- [x] Dependency checks
- [x] Virtualization capability check
- [x] GPU capability check
- [x] Disk-space check
- [x] First-run wizard
- [x] Download default Android image
- [x] Create first instance
- [x] Repair installation
- [x] Uninstaller

---

# Phase 40 - First-Run Wizard

- [x] Detect CPU
- [x] Detect VT-x/AMD-V
- [x] Detect Hyper-V/WHPX
- [x] Detect KVM
- [x] Detect GPU
- [x] Detect Vulkan
- [x] Detect hardware codecs
- [x] Recommend RAM allocation
- [x] Recommend CPU allocation
- [x] Recommend renderer
- [x] Recommend Android version
- [x] Create gaming profile

---

# Phase 41 - Remote Access

- [x] Self-hosted remote signalling node
- [x] Docker deployment
- [x] Expiring invite codes
- [x] Remote node shared-secret authentication
- [x] Per-session host tokens
- [x] Browser/mobile viewer
- [x] Host viewer approval
- [x] Deny viewer
- [x] Revoke viewer
- [x] View-only permission
- [x] Android control permission
- [x] WebRTC signalling
- [x] STUN support
- [x] Optional TURN relay deployment
- [x] Touch/tap forwarding
- [x] Swipe forwarding
- [x] Android navigation key forwarding
- [x] Keyboard forwarding
- [x] ADB-backed remote Android input
- [x] Explicit host screen/window sharing
- [x] Direct emulator framebuffer WebRTC capture
- [x] Direct Android audio capture
- [x] Adaptive bitrate
- [x] 30/60/90/120 FPS remote presets
- [x] Clipboard synchronization
- [x] Encrypted remote file transfer
- [x] Gamepad forwarding
- [x] Remote microphone
- [x] Remote virtual camera
- [x] Persistent trusted-device pairing
- [x] Optional owner-configured unattended access
- [x] Multi-region signalling nodes
- [x] Connection quality statistics
- [x] Native Android viewer app
- [x] PWA install support

---

# Long-Term Ideas

- [x] Cloud Android nodes
- [x] Remote control from browser
- [x] Android instance streaming
- [x] Remote ADB management
- [x] Plugin system
- [x] Custom renderer plugins
- [x] Custom AI plugins
- [x] Community device profiles
- [x] Community game profiles
- [x] Community AI skills
- [x] Built-in performance benchmark
- [x] Android update manager
- [x] Virtual webcam passthrough
- [x] USB passthrough
- [x] Bluetooth passthrough
- [x] Controller hot-plug
- [x] Multiple virtual displays
- [x] Foldable hinge simulation
- [x] Desktop mode support
- [x] Android Automotive profile
- [x] Android TV profile

---

# Initial MVP Definition

The first usable release should include:

- [x] Windows launcher
- [x] Android 16 x86_64 boot
- [x] GPU acceleration
- [x] Audio
- [x] Internet
- [x] Mouse/touch
- [x] Keyboard
- [x] Portrait/landscape rotation
- [x] ADB
- [x] ADB file push/pull
- [x] Drag-and-drop APK installation
- [x] PC-to-Android file transfer
- [x] Android-to-PC file transfer
- [x] Basic device profiles
- [x] Basic multi-instance support
- [x] Snapshots
- [x] 60 FPS mode
- [x] Root-capable developer image
- [x] Basic framebuffer capture
- [x] Basic AI virtual tap/swipe API

After the MVP is stable, development should focus on:

1. ARM application compatibility
2. 120+ FPS gaming
3. Google Play-compatible images
4. Advanced keymapping
5. Gamepad support
6. AI game skills
7. MediaCodec/FFmpeg acceleration
8. Streaming/recording
9. Android 9-15 image support
10. Advanced root/developer tooling
