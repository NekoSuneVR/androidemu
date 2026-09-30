# NekoDroid Remote Android Client

This is the installable Android companion for NekoDroid Remote.

It is intentionally lightweight: the native app asks for the self-hosted NekoDroid Remote node URL and an invite code, then opens the existing WebRTC viewer/control interface in Android WebView.

## Build locally

Requirements:

- JDK 17
- Android SDK 35
- Gradle 8.10.2

Build:

```bash
gradle assembleDebug
```

The installable APK is created at:

```text
app/build/outputs/apk/debug/app-debug.apk
```

GitHub Actions builds this through the manual release workflow.
