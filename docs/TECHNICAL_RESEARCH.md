# NekoDroid Technical Research Decisions

This document records completed investigation tasks from TODO.md. It does not claim that the corresponding runtime integrations are implemented.

## Hyper-V-compatible acceleration

NekoDroid uses QEMU WHPX on Windows as the practical Hyper-V-compatible acceleration path. WHPX can coexist with the Windows hypervisor stack and is already probed by the runtime. Native Hyper-V VM management is not required for the initial architecture; the launcher should continue selecting WHPX when QEMU advertises it and fall back clearly when unavailable.

## gfxstream

gfxstream is a candidate Android guest graphics path because it is designed around Android/emulator graphics virtualization. Integrating it into NekoDroid would require a compatible Android guest image, host renderer components and a tested transport path; it cannot be enabled merely by adding a QEMU flag. The implementation task remains separate from this investigation.

## ARM64 guest mode

ARM64 guests are a possible compatibility route, but they are not a replacement for hardware-accelerated x86_64 guests on common x86_64 Windows/Linux hosts. On x86_64 hosts an ARM64 system guest normally requires CPU emulation and can be much slower. NekoDroid should prefer native x86_64 guest execution plus a legally distributable Android native bridge where possible, while keeping ARM64 guest mode experimental.

## Clipboard file transfer

Clipboard file transfer should not overload the text clipboard protocol. The preferred design is a small transfer manifest containing file names, sizes and transfer identifiers, with the actual bytes moved through the existing host/ADB transfer layer or a future encrypted remote-transfer channel. Text clipboard synchronization and file transfer should remain separately permissioned.

## Memory ballooning

VirtIO ballooning can be useful for reclaiming guest memory, but Android guest support and latency need testing before enabling it by default. NekoDroid should expose ballooning only when the guest image has the required virtio driver and should keep fixed RAM allocation as the predictable default for gaming.

## NDI-style output

NDI output should be treated as an optional plugin/integration rather than a core dependency because SDK/distribution terms and platform support differ. The core streaming layer should expose reusable raw/encoded video and audio frames so an optional NDI-compatible module can consume them later.

## Spout-style output

Spout is Windows/GPU focused. A future output module should share the emulator render texture directly where possible rather than round-tripping through CPU memory. This belongs behind a platform-specific output interface; non-Windows builds should not depend on it.

## GrapheneOS-derived testing images

GrapheneOS security properties depend on supported Pixel hardware and hardware-backed features that a generic VM cannot reproduce. NekoDroid may support user-supplied compatible research/testing images through the generic secure-image/GSI path, but must display the actual verified-boot/security state and must not claim Pixel identity or hardware-backed attestation.
