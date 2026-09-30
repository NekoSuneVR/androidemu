# NekoDroid TODO

This file tracks the planned implementation of NekoDroid.

---

# Phase 0 - Project Foundation

- [ ] Choose final project name
- [x] Create repository
- [ ] Add open-source license
- [x] Add `README.md`
- [x] Add `TODO.md`
- [ ] Add contributing guide
- [ ] Add code of conduct
- [x] Set up CI
- [ ] Set up release builds
- [ ] Create Windows build pipeline
- [ ] Create Linux build pipeline
- [ ] Decide minimum Windows version
- [ ] Decide minimum Linux requirements
- [ ] Create project branding/icons

---

# Phase 1 - Desktop Launcher

- [x] Create Tauri application
- [x] Add React frontend
- [x] Add Rust backend
- [x] Create dark modern UI
- [x] Create Home page
- [ ] Create Game Library page
- [ ] Create Android Instances page
- [ ] Create Device Profiles page
- [ ] Create Media Tools page
- [ ] Create Developer Tools page
- [ ] Create AI page
- [ ] Create Settings page
- [ ] Add update system
- [ ] Add logs viewer
- [ ] Add crash reports stored locally
- [ ] Add portable mode

---

# Phase 2 - Virtualization Core

- [ ] Integrate QEMU
- [ ] Add Windows WHPX support
- [ ] Investigate Hyper-V-compatible acceleration
- [ ] Add Linux KVM support
- [ ] Add VirtIO block devices
- [ ] Add VirtIO networking
- [ ] Add VirtIO input
- [ ] Add VirtIO audio
- [ ] Add QCOW2 storage
- [ ] Add dynamically expanding userdata disks
- [ ] Add snapshot support
- [ ] Add clone support
- [ ] Add VM pause/resume
- [ ] Add headless mode
- [ ] Add off-screen rendering

---

# Phase 3 - Android Image Manager

- [ ] Design image manifest format
- [ ] Add Android image downloader
- [ ] Add checksum verification
- [ ] Add image installation
- [ ] Add image removal
- [ ] Add image update handling
- [ ] Add custom image import
- [ ] Add custom GSI import
- [ ] Add image repair

## Android Versions

- [ ] Android 9
- [ ] Android 10
- [ ] Android 11
- [ ] Android 12
- [ ] Android 12L
- [ ] Android 13
- [ ] Android 14
- [ ] Android 15
- [ ] Android 16
- [ ] Future Android versions

---

# Phase 4 - Android Boot

- [ ] Boot Android 16 x86_64
- [ ] Display boot animation
- [ ] Reach Android launcher
- [ ] Working system UI
- [ ] Working storage
- [ ] Working network
- [ ] Working audio
- [ ] Working clock/timezone
- [ ] Working clipboard
- [ ] Working Android shutdown
- [ ] Working Android restart
- [ ] Detect Android boot completion
- [ ] Detect Android crashes

---

# Phase 5 - Device Profiles

- [ ] Create device profile format
- [ ] Phone profile
- [ ] Gaming phone profile
- [ ] Tablet profile
- [ ] Large tablet profile
- [ ] Foldable profile
- [ ] Custom profile

## Profile Settings

- [ ] Resolution
- [ ] DPI
- [ ] Refresh rate
- [ ] CPU count
- [ ] RAM
- [ ] Storage size
- [ ] Touch point count
- [ ] Wi-Fi capability
- [ ] Bluetooth capability
- [ ] GPS capability
- [ ] Camera configuration
- [ ] Microphone
- [ ] Accelerometer
- [ ] Gyroscope
- [ ] Compass
- [ ] Light sensor
- [ ] Proximity sensor
- [ ] Battery state
- [ ] Charging state
- [ ] Telephony availability
- [ ] Tablet resource configuration

---

# Phase 6 - Screen Rotation

- [ ] Automatic orientation
- [ ] Portrait
- [ ] Reverse portrait
- [ ] Landscape
- [ ] Reverse landscape
- [ ] Rotate-left button
- [ ] Rotate-right button
- [ ] Rotation hotkey
- [ ] Update Android orientation at runtime
- [ ] Resize emulator window after rotation
- [ ] Update touch coordinate transforms
- [ ] Update AI coordinate transforms
- [ ] Update keymapping overlays
- [ ] Update controller overlays
- [ ] Update recording dimensions
- [ ] Update streaming dimensions
- [ ] Persist orientation per app
- [ ] Auto-detect app preferred orientation

---

# Phase 7 - Graphics Acceleration

- [ ] Vulkan host detection
- [ ] OpenGL host detection
- [ ] DirectX capability detection
- [ ] gfxstream investigation
- [ ] ANGLE integration
- [ ] VirGL integration
- [ ] Vulkan mode
- [ ] OpenGL mode
- [ ] DirectX translation mode
- [ ] Software fallback
- [ ] Shader cache
- [ ] ASTC support
- [ ] ASTC texture cache
- [ ] Frame pacing
- [ ] VSync
- [ ] Triple buffering
- [ ] Dynamic resolution
- [ ] GPU capability page

---

# Phase 8 - Frame Rate

- [ ] 30 FPS
- [ ] 60 FPS
- [ ] 90 FPS
- [ ] 120 FPS
- [ ] 144 FPS
- [ ] 165 FPS
- [ ] 240 FPS
- [ ] Unlimited FPS
- [ ] Per-game FPS setting
- [ ] Frame limiter
- [ ] FPS overlay
- [ ] Frame time graph
- [ ] CPU/GPU usage overlay

---

# Phase 9 - ARM Compatibility

- [ ] Inspect APK ABI
- [ ] Prefer x86_64 when available
- [ ] Native bridge architecture
- [ ] Investigate `libndk_translation`
- [ ] ARM64 application compatibility
- [ ] ARMv7 compatibility
- [ ] ARM64 guest mode research
- [ ] Per-app compatibility mode
- [ ] Compatibility diagnostics

---

# Phase 10 - Google Services

- [ ] Create modular GMS provider interface
- [ ] Support legally obtained Google-compatible images
- [ ] Play Store launch support
- [ ] Google account login testing
- [ ] Google Play Services testing
- [ ] microG profile
- [ ] Custom GApps profile
- [ ] Explain certification status in UI
- [ ] Display Play Integrity/security limitations
- [ ] Never falsely report hardware-backed attestation

---

# Phase 11 - GrapheneOS / Custom Secure Images

- [ ] Investigate technically compatible GrapheneOS-derived testing images
- [ ] Add generic custom secure-image profile
- [ ] Add custom GSI boot support
- [ ] Display actual verified boot state
- [ ] Display actual security state
- [ ] Display missing hardware-backed features
- [ ] Do not claim genuine Pixel hardware identity
- [ ] Do not spoof hardware-backed attestation

---

# Phase 12 - ADB

- [ ] Bundle/platform-tools manager
- [ ] Toggle ADB on/off
- [ ] ADB over localhost
- [ ] Optional ADB over LAN
- [ ] ADB authentication
- [ ] Per-instance ADB ports
- [ ] Connect/disconnect button
- [ ] Built-in ADB terminal
- [ ] `adb shell`
- [ ] `adb install`
- [ ] `adb uninstall`
- [ ] `adb push`
- [ ] `adb pull`
- [ ] `adb forward`
- [ ] `adb reverse`
- [ ] `adb reboot`
- [ ] Reboot recovery
- [ ] Screenshot
- [ ] Screen recording
- [ ] Device information
- [ ] Package manager GUI
- [ ] Process viewer
- [ ] Logcat viewer

---

# Phase 13 - PC to Android File Transfer

- [ ] Host-to-Android file transfer UI
- [ ] Android-to-host file transfer UI
- [ ] Drag-and-drop into Android
- [ ] Default drop folder `/sdcard/Download/`
- [ ] Custom destination selection
- [ ] File transfer progress
- [ ] Cancel transfer
- [ ] Transfer retry
- [ ] Large-file support
- [ ] Multiple-file transfer
- [ ] Folder upload
- [ ] Folder download
- [ ] ADB push backend
- [ ] ADB pull backend
- [ ] Shared-folder alternative
- [ ] Clipboard file transfer research

## APK Installation

- [ ] Drag-and-drop APK install
- [ ] Select APK from PC
- [ ] Batch APK installation
- [ ] Split APK support
- [ ] APKS bundle support
- [ ] XAPK support
- [ ] OBB/data handling
- [ ] Show package name/version before install
- [ ] Install progress
- [ ] Install result/error display

---

# Phase 14 - Root

- [ ] Standard mode
- [ ] Developer mode
- [ ] ADB root mode
- [ ] Full root mode
- [ ] Root warning UI
- [ ] Root shell
- [ ] `su` support for compatible images
- [ ] Writable system overlay
- [ ] Root filesystem browser
- [ ] Root-on-next-boot
- [ ] Separate rooted snapshot
- [ ] Separate clean snapshot
- [ ] Factory reset
- [ ] Explain app compatibility risks
- [ ] Explain Play Integrity impact

---

# Phase 15 - Android File Manager

- [ ] Browse `/sdcard`
- [ ] Browse `/storage`
- [ ] Upload
- [ ] Download
- [ ] Rename
- [ ] Delete
- [ ] Copy
- [ ] Move
- [ ] Create folder
- [ ] File properties
- [ ] Storage usage
- [ ] Permissions viewer
- [ ] Root filesystem mode
- [ ] Search files

---

# Phase 16 - Virtual Touch

- [ ] Tap
- [ ] Double tap
- [ ] Hold
- [ ] Swipe
- [ ] Drag
- [ ] Multi-touch
- [ ] Pinch
- [ ] Zoom
- [ ] Virtual joystick
- [ ] Touch overlays
- [ ] Touch debugging overlay

---

# Phase 17 - Keyboard and Mouse Mapping

- [ ] WASD joystick mapping
- [ ] Key-to-tap mapping
- [ ] Key-to-hold mapping
- [ ] Mouse-look
- [ ] Mouse lock
- [ ] Left-click mapping
- [ ] Right-click mapping
- [ ] Scroll mapping
- [ ] Per-game keymaps
- [ ] Import/export keymaps
- [ ] Keymap editor
- [ ] Visual overlay editor
- [ ] Hotkey to show/hide controls

---

# Phase 18 - Gamepad

- [ ] SDL gamepad support
- [ ] Xbox controller
- [ ] PlayStation controller
- [ ] Generic controller
- [ ] Virtual Android gamepad
- [ ] Analog sticks
- [ ] Triggers
- [ ] Vibration
- [ ] Controller mapping UI
- [ ] Per-game mappings

---

# Phase 19 - Virtual Sensors

- [ ] Accelerometer API
- [ ] Gyroscope API
- [ ] Compass API
- [ ] GPS API
- [ ] Light sensor
- [ ] Proximity sensor
- [ ] Battery simulation
- [ ] Charging simulation
- [ ] Sensor control UI
- [ ] Sensor automation API
- [ ] Map/location test interface

---

# Phase 20 - NekoAI Core

- [ ] AI subsystem interface
- [ ] AI enable/disable toggle
- [ ] Local AI mode
- [ ] Remote Ollama mode
- [ ] OpenAI-compatible endpoint mode
- [ ] Model selection
- [ ] Vision model selection
- [ ] Object detector selection
- [ ] OCR engine
- [ ] Speech recognition
- [ ] TTS
- [ ] AI logs
- [ ] AI performance limits
- [ ] AI emergency stop

---

# Phase 21 - AI Framebuffer Access

- [ ] Direct framebuffer capture
- [ ] Shared memory capture
- [ ] Avoid high-frequency ADB screenshots
- [ ] Region-of-interest capture
- [ ] Frame-difference detection
- [ ] AI frame-rate limiter
- [ ] GPU-to-AI low-copy path
- [ ] Headless AI support
- [ ] Off-screen AI support

---

# Phase 22 - AI Virtual Controls

- [ ] AI tap
- [ ] AI hold
- [ ] AI swipe
- [ ] AI drag
- [ ] AI multi-touch
- [ ] AI virtual joystick
- [ ] AI virtual gamepad
- [ ] AI keyboard input
- [ ] AI text input
- [ ] AI gyro input
- [ ] AI accelerometer input
- [ ] AI action queue
- [ ] AI action cancellation
- [ ] AI input visualizer

The AI should use virtual Android controls instead of moving the host operating-system mouse.

---

# Phase 23 - AI Game Skills

- [ ] Define skill manifest
- [ ] Generic Android skill
- [ ] Package-name detection
- [ ] Orientation detection
- [ ] Control definitions
- [ ] UI region definitions
- [ ] Game-state extraction
- [ ] Per-game prompts
- [ ] Skill import/export
- [ ] Skill marketplace/repository format
- [ ] Local custom skills

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

- [ ] Manual
- [ ] Assistant
- [ ] Accessibility
- [ ] Full automation
- [ ] Voice-command mode
- [ ] Inventory helper
- [ ] Quest helper
- [ ] UI helper
- [ ] Repetitive-task helper
- [ ] Game-specific automation policy
- [ ] Per-game AI permissions

---

# Phase 25 - MediaCodec Bridge

- [ ] Detect Android MediaCodec requests
- [ ] Create host codec bridge
- [ ] H.264 decode
- [ ] H.264 encode
- [ ] H.265 decode
- [ ] H.265 encode
- [ ] VP8 decode
- [ ] VP9 decode
- [ ] AV1 decode
- [ ] AV1 encode where available
- [ ] Audio decode
- [ ] Audio encode
- [ ] Software fallback
- [ ] MediaCodec capability reporting

---

# Phase 26 - FFmpeg Engine

- [ ] Bundle/manage FFmpeg
- [ ] Detect FFmpeg capabilities
- [ ] Hardware decode
- [ ] Hardware encode
- [ ] Software decode
- [ ] Software encode
- [ ] Progress reporting
- [ ] Cancel jobs
- [ ] Queue jobs
- [ ] Batch processing

---

# Phase 27 - Hardware Video Acceleration

## NVIDIA

- [ ] NVDEC detection
- [ ] NVENC detection
- [ ] H.264 capability
- [ ] HEVC capability
- [ ] AV1 capability on supported GPUs

## Intel

- [ ] Quick Sync detection
- [ ] H.264
- [ ] HEVC
- [ ] VP9
- [ ] AV1 where supported

## AMD

- [ ] AMF detection
- [ ] VCN detection
- [ ] H.264
- [ ] HEVC
- [ ] AV1 where supported

---

# Phase 28 - Built-In Media Tools

- [ ] Video converter
- [ ] Audio converter
- [ ] Video compressor
- [ ] Audio extraction
- [ ] Remux
- [ ] Resolution conversion
- [ ] Frame-rate conversion
- [ ] Hardware transcoding
- [ ] Batch converter
- [ ] Drag-and-drop input

---

# Phase 29 - Codec Support

## Video

- [ ] H.264 / AVC
- [ ] H.265 / HEVC
- [ ] VP8
- [ ] VP9
- [ ] AV1
- [ ] MPEG-4
- [ ] MPEG-2 where required

## Audio

- [ ] AAC
- [ ] MP3
- [ ] Opus
- [ ] Vorbis
- [ ] FLAC
- [ ] PCM/WAV
- [ ] ALAC where available

---

# Phase 30 - Recording

- [ ] Direct framebuffer recording
- [ ] H.264 recording
- [ ] HEVC recording
- [ ] Hardware encoder selection
- [ ] Audio recording
- [ ] Microphone mixing
- [ ] Android audio capture
- [ ] Recording quality presets
- [ ] Recording FPS selection
- [ ] Screenshot hotkey

---

# Phase 31 - Streaming

- [ ] OBS-friendly capture
- [ ] Virtual camera output
- [ ] RTMP output
- [ ] SRT output
- [ ] WebRTC output
- [ ] NDI-style output research
- [ ] Spout-style output research
- [ ] Audio streaming
- [ ] Rotated-stream handling

---

# Phase 32 - Multi-Instance

- [ ] Instance creation
- [ ] Instance deletion
- [ ] Instance rename
- [ ] Instance clone
- [ ] Concurrent instances
- [ ] Separate userdata
- [ ] Separate Android versions
- [ ] Separate device profiles
- [ ] Separate root setting
- [ ] Separate ADB ports
- [ ] Separate AI settings
- [ ] Separate keymaps
- [ ] Separate storage
- [ ] Separate snapshots
- [ ] CPU/RAM resource limits

---

# Phase 33 - Snapshot Manager

- [ ] Create snapshot
- [ ] Restore snapshot
- [ ] Delete snapshot
- [ ] Rename snapshot
- [ ] Snapshot description
- [ ] Auto snapshot before root
- [ ] Auto snapshot before Android update
- [ ] Clean snapshot
- [ ] Rooted snapshot
- [ ] Snapshot storage cleanup

---

# Phase 34 - Developer Tools

- [ ] ADB terminal
- [ ] Root terminal
- [ ] Logcat
- [ ] Kernel log
- [ ] Package viewer
- [ ] Activity viewer
- [ ] Process viewer
- [ ] Service viewer
- [ ] Network connections
- [ ] Port forwarding
- [ ] Android property viewer
- [ ] Build property viewer
- [ ] Storage inspector
- [ ] SurfaceFlinger information

---

# Phase 35 - Automation API

- [ ] REST server
- [ ] WebSocket server
- [ ] API authentication
- [ ] localhost-only default
- [ ] Instance API
- [ ] Start/stop API
- [ ] Screenshot/frame API
- [ ] Tap API
- [ ] Swipe API
- [ ] Keyboard API
- [ ] Gamepad API
- [ ] File push API
- [ ] File pull API
- [ ] APK install API
- [ ] ADB API
- [ ] Rotation API
- [ ] Reboot API
- [ ] AI API
- [ ] Log stream API
- [ ] Audio stream API

---

# Phase 36 - Security

- [ ] ADB off by default
- [ ] Root off by default
- [ ] AI control off by default
- [ ] LAN ADB off by default
- [ ] API localhost-only by default
- [ ] Require user approval for dangerous actions
- [ ] Root warning
- [ ] Network exposure warning
- [ ] Instance isolation
- [ ] Secure ADB keys
- [ ] Secure configuration storage
- [ ] Sensitive log filtering

---

# Phase 37 - Performance Tuning

- [ ] CPU affinity
- [ ] Huge pages
- [ ] I/O tuning
- [ ] Memory ballooning research
- [ ] RAM compression
- [ ] Disk cache
- [ ] Shader cache tuning
- [ ] Audio latency tuning
- [ ] Input latency tuning
- [ ] Frame pacing tuning
- [ ] Startup optimization
- [ ] Background service minimization
- [ ] CPU usage overlay
- [ ] GPU usage overlay
- [ ] RAM usage overlay

---

# Phase 38 - Game Compatibility

- [ ] Package-specific settings
- [ ] Per-game renderer selection
- [ ] Per-game Android version
- [ ] Per-game orientation
- [ ] Per-game DPI
- [ ] Per-game FPS
- [ ] Per-game keymap
- [ ] Per-game AI skill
- [ ] Compatibility database
- [ ] Known-issues system
- [ ] Crash diagnostics

---

# Phase 39 - Installer

- [ ] Windows installer
- [ ] Linux installer
- [ ] Dependency checks
- [ ] Virtualization capability check
- [ ] GPU capability check
- [ ] Disk-space check
- [ ] First-run wizard
- [ ] Download default Android image
- [ ] Create first instance
- [ ] Repair installation
- [ ] Uninstaller

---

# Phase 40 - First-Run Wizard

- [ ] Detect CPU
- [ ] Detect VT-x/AMD-V
- [ ] Detect Hyper-V/WHPX
- [ ] Detect KVM
- [ ] Detect GPU
- [ ] Detect Vulkan
- [ ] Detect hardware codecs
- [ ] Recommend RAM allocation
- [ ] Recommend CPU allocation
- [ ] Recommend renderer
- [ ] Recommend Android version
- [ ] Create gaming profile

---

# Long-Term Ideas

- [ ] Cloud Android nodes
- [ ] Remote control from browser
- [ ] Android instance streaming
- [ ] Remote ADB management
- [ ] Plugin system
- [ ] Custom renderer plugins
- [ ] Custom AI plugins
- [ ] Community device profiles
- [ ] Community game profiles
- [ ] Community AI skills
- [ ] Built-in performance benchmark
- [ ] Android update manager
- [ ] Virtual webcam passthrough
- [ ] USB passthrough
- [ ] Bluetooth passthrough
- [ ] Controller hot-plug
- [ ] Multiple virtual displays
- [ ] Foldable hinge simulation
- [ ] Desktop mode support
- [ ] Android Automotive profile
- [ ] Android TV profile

---

# Initial MVP Definition

The first usable release should include:

- [ ] Windows launcher
- [ ] Android 16 x86_64 boot
- [ ] GPU acceleration
- [ ] Audio
- [ ] Internet
- [ ] Mouse/touch
- [ ] Keyboard
- [ ] Portrait/landscape rotation
- [ ] ADB
- [ ] ADB file push/pull
- [ ] Drag-and-drop APK installation
- [ ] PC-to-Android file transfer
- [ ] Android-to-PC file transfer
- [ ] Basic device profiles
- [ ] Basic multi-instance support
- [ ] Snapshots
- [ ] 60 FPS mode
- [ ] Root-capable developer image
- [ ] Basic framebuffer capture
- [ ] Basic AI virtual tap/swipe API

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
