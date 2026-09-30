export type InstanceStatus = "stopped" | "starting" | "running" | "paused" | "error";

export interface AndroidInstance {
  id: string;
  name: string;
  androidVersion: string;
  profile: string;
  status: InstanceStatus;
  cpuCores: number;
  ramMb: number;
  adbPort: number;
  rootMode: "standard" | "developer" | "adb-root" | "full-root";
  imagePath?: string | null;
  processId?: number | null;
}

export interface CreateInstanceRequest {
  name: string;
  androidVersion: string;
  profile: string;
  cpuCores: number;
  ramMb: number;
  adbPort: number;
  rootMode: AndroidInstance["rootMode"];
  imagePath?: string | null;
}

export interface QemuInfo {
  found: boolean;
  executable?: string | null;
  version?: string | null;
  accelerators: string[];
}

export interface HostCapabilities {
  os: string;
  arch: string;
  accelerator: string;
  acceleratorAvailable: boolean;
  virtualizationNote: string;
  qemu: QemuInfo;
}

export interface RuntimeActionResult {
  instanceId: string;
  status: InstanceStatus;
  message: string;
  processId?: number | null;
}

export interface DeviceProfile {
  id: string;
  name: string;
  width: number;
  height: number;
  dpi: number;
  refreshRate: number;
  defaultCpuCores: number;
  defaultRamMb: number;
  touchPoints: number;
  telephony: boolean;
  formFactor: string;
}

export interface AndroidImageManifest {
  id: string;
  name: string;
  androidVersion: string;
  api: number;
  architecture: string;
  imageType: string;
  disk: string;
  diskFormat: "qcow2" | "raw";
  recommended: boolean;
  notes?: string | null;
}

export interface InstalledImage {
  manifest: AndroidImageManifest;
  directory: string;
  diskPath: string;
  valid: boolean;
  validationError?: string | null;
}

export interface AdbInfo {
  found: boolean;
  executable?: string | null;
  version?: string | null;
}

export interface AdbResult {
  success: boolean;
  exitCode?: number | null;
  stdout: string;
  stderr: string;
}
