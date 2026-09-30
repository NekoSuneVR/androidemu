# Contributing to NekoDroid

Thanks for helping build NekoDroid.

## Development flow

1. Fork or branch from `main`.
2. Keep one feature area per branch where practical.
3. Run the frontend build and Rust checks before opening a pull request.
4. Do not mark roadmap items complete until the implementation actually exists.
5. Keep Windows and Linux behavior separated behind runtime capability checks.

## Local checks

```bash
npm install
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

For full desktop testing:

```bash
npm run tauri dev
```

## Runtime principles

NekoDroid should:

- keep ADB bound to localhost by default;
- keep root disabled by default;
- keep AI control disabled by default;
- report actual virtualization/security state;
- avoid pretending a VM has hardware-backed Pixel security or attestation;
- keep per-instance data isolated;
- validate paths and image manifests before launching QEMU;
- write useful runtime logs when QEMU fails.

## Code organization

```text
src/                     React / TypeScript launcher
src/components/          Launcher feature views
src-tauri/src/runtime.rs QEMU discovery and process lifecycle
src-tauri/src/storage.rs Persistent instance configuration
src-tauri/src/images.rs  Android image registry
src-tauri/src/profiles.rs Built-in device profiles
src-tauri/src/adb.rs     Android Debug Bridge integration
```

## Pull requests

Please explain:

- what changed;
- which platform(s) were tested;
- which roadmap entries the change satisfies;
- any known limitations;
- whether the change affects root, ADB, networking or security behavior.

Do not include proprietary Google components, device keys, private signing keys, copyrighted Android images, or credentials in commits.
