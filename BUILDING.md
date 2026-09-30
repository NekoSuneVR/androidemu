# Building NekoDroid

NekoDroid is currently an early development build. The launcher, persistent instance model, QEMU process manager, image registry, device profiles and localhost ADB tools are implemented on the development branch.

Android 16 boot has **not** been marked complete yet. The current runtime can launch a bootable x86_64 raw/QCOW2 disk through QEMU; Android-specific AOSP partition boot support is the next runtime milestone.

## Prerequisites

### Common

- Node.js 22+
- npm
- Rust stable toolchain
- QEMU with `qemu-system-x86_64`
- Android Platform Tools / `adb`

### Windows

Recommended:

- Windows 10/11 x64
- CPU virtualization enabled in UEFI/BIOS
- Windows Hypervisor Platform enabled
- A QEMU build that exposes the `whpx` accelerator
- WebView2 runtime

Check QEMU:

```powershell
qemu-system-x86_64 --version
qemu-system-x86_64 -accel help
```

The accelerator list should include:

```text
whpx
tcg
```

Check ADB:

```powershell
adb version
```

### Linux

Recommended:

- Recent x86_64 distribution
- KVM enabled
- User permission to access `/dev/kvm`
- QEMU x86 system package
- WebKitGTK 4.1 development packages required by Tauri

Ubuntu/Debian example:

```bash
sudo apt update
sudo apt install -y \
  qemu-system-x86 qemu-utils adb \
  libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

test -e /dev/kvm && echo "KVM device exists"
qemu-system-x86_64 -accel help
adb version
```

Your user may need membership of the `kvm` group:

```bash
sudo usermod -aG kvm "$USER"
```

Sign out and back in after changing group membership.

## Install JavaScript dependencies

```bash
npm install
```

## Run in development mode

```bash
npm run tauri dev
```

## Build the frontend only

```bash
npm run build
```

## Rust check

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

## Build desktop bundle

```bash
npm run tauri build
```

## Runtime data

NekoDroid stores runtime data in the Tauri application data directory.

The layout is designed as:

```text
NekoDroid app data/
├── images/
│   └── <image-id>/
│       ├── manifest.json
│       └── android.qcow2
└── instances/
    └── <instance-id>/
        ├── config.json
        ├── logs/
        │   ├── qemu.out.log
        │   └── qemu.err.log
        └── snapshots/
```

## Registering an Android image

Open **Android Images** in the launcher.

The current image registry accepts:

- x86_64
- QCOW2
- raw / IMG

The image is copied into NekoDroid's application data directory.

A registered image can then be selected while creating an instance.

## ADB

Each instance receives its own localhost ADB port.

Example:

```text
Gaming -> 127.0.0.1:5555
Testing -> 127.0.0.1:5556
```

The QEMU user-network backend forwards that host port to guest TCP 5555.

The Developer Tools page currently provides:

- connect
- disconnect
- get-state
- shell command execution
- APK install
- push
- pull

LAN ADB exposure is not enabled by the runtime.

## Current runtime limitation

The first QEMU runtime supports a **bootable disk image**. AOSP emulator-style Android builds often use separate kernel, ramdisk, system, vendor and userdata images.

The next Android boot layer should add an image manifest similar to:

```json
{
  "id": "android-16-aosp-x86_64",
  "androidVersion": "16",
  "api": 36,
  "architecture": "x86_64",
  "bootMode": "aosp",
  "kernel": "kernel-ranchu",
  "ramdisk": "ramdisk.img",
  "system": "system.img",
  "vendor": "vendor.img",
  "userdata": "userdata.qcow2"
}
```

That work should be completed before the roadmap item **Boot Android 16 x86_64** is checked.


## Portable mode

NekoDroid normally stores persistent data in the operating system application-data directory.

To keep instances, profiles, settings, logs, snapshots and imported images beside the NekoDroid executable instead, set:

### Windows PowerShell

```powershell
$env:NEKODROID_PORTABLE = "1"
.\NekoDroid.exe
```

### Linux

```bash
NEKODROID_PORTABLE=1 ./nekodroid
```

Portable data is stored in:

```text
NekoDroidData/
```

beside the running executable.


## Dynamically expanding instance storage

Each NekoDroid instance uses a private QCOW2 runtime overlay backed by the registered Android base image.

The overlay starts small and grows as the guest writes data, so separate instances do not duplicate the entire base image up front. The base image remains read-only from NekoDroid's point of view, while instance changes are stored under:

```text
instances/<instance-id>/disks/runtime.qcow2
```

Factory reset deletes this writable overlay and recreates it from the base image on the next start.


## Supported desktop baseline

### Windows

NekoDroid's current Windows baseline is:

- Windows 10 version 2004 or newer
- x86_64 CPU
- Windows Hypervisor Platform enabled for WHPX acceleration
- WebView2 runtime
- QEMU with WHPX support
- Android platform-tools for ADB features
- FFmpeg for Media Tools

Older Windows versions may be able to launch the Tauri shell, but they are outside the supported NekoDroid virtualization baseline because this project relies on the WHPX path.

### Linux

NekoDroid's current Linux baseline is:

- x86_64 Linux
- Ubuntu 22.04 / Debian 12 or newer as the packaging baseline, or an equivalent distribution with WebKitGTK 4.1
- KVM with access to `/dev/kvm`
- QEMU x86_64
- Android platform-tools for ADB features
- FFmpeg for Media Tools
- WebKitGTK 4.1 and the desktop libraries required by Tauri 2

The Linux release workflow produces Debian and AppImage bundles. Building on significantly newer distributions can raise the required glibc version, so release builds should stay on the oldest supported baseline when compatibility matters.
