# NekoDroid Guest Bridge

Optional guest APK for capabilities Android's standard ADB `input` command cannot provide.

Current bridge:
- simultaneous multi-touch gesture injection
- pinch / zoom gesture injection

Install the APK in the guest and explicitly enable **NekoDroid Guest Bridge** under Android Accessibility before using these controls.

Sensor broadcasts use the same `uk.co.nekosunevr.nekodroid.bridge.*` namespace as a stable protocol for future privileged/system-image sensor providers. The accessibility APK does not falsely claim it can replace Android's hardware SensorManager values.
