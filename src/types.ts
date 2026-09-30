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
  adbEnabled: boolean;
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
  adbEnabled: boolean;
  rootMode: AndroidInstance["rootMode"];
  imagePath?: string | null;
}

export interface UpdateInstanceRequest {
  name: string;
  adbEnabled: boolean;
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


export interface AiSettings {
  enabled: boolean;
  mode: "local" | "ollama" | "openai-compatible";
  endpoint: string;
  model: string;
  visionModel: string;
  detectorModel: string;
  maxActionsPerMinute: number;
  maxCaptureFps: number;
}


export interface RuntimeLogs {
  stdout: string;
  stderr: string;
  crashReport: string;
}
