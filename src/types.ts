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
  headless: boolean;
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
  headless: boolean;
  rootMode: AndroidInstance["rootMode"];
  imagePath?: string | null;
}

export interface UpdateInstanceRequest {
  name: string;
  adbEnabled: boolean;
  headless: boolean;
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
  sha256?: string | null;
  sourceUrl?: string | null;
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


export interface SnapshotInfo {
  id: string;
  name: string;
  description: string;
  createdAt: number;
}


export interface FfmpegInfo {
  found: boolean;
  executable?: string | null;
  version?: string | null;
  hwaccels: string[];
  hardwareEncoders: string[];
  hardwareDecoders: string[];
}

export interface MediaResult {
  success: boolean;
  exitCode?: number | null;
  stdout: string;
  stderr: string;
  outputPath: string;
}


export interface AppSettings {
  defaultAndroidVersion: string;
  defaultProfile: string;
  defaultAdbEnabled: boolean;
  defaultHeadless: boolean;
  confirmDangerousActions: boolean;
  apiEnabled: boolean;
  apiPort: number;
  apiToken: string;
  firstRunCompleted: boolean;
}


export interface SystemReadiness {
  cpuModel: string;
  logicalCores: number;
  totalMemoryMb?: number | null;
  virtualizationAvailable: boolean;
  virtualizationDetail: string;
  gpuNames: string[];
  vulkanAvailable: boolean;
  vulkanDetail: string;
  hardwareEncoders: string[];
  freeDiskMb?: number | null;
  qemuFound: boolean;
  adbFound: boolean;
  ffmpegFound: boolean;
  recommendedCpuCores: number;
  recommendedRamMb: number;
  recommendedAndroidVersion: string;
  recommendedRenderer: string;
}


export interface AndroidFileEntry {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  permissions: string;
  modified: number;
}


export interface MediaJobRequest {
  input: string;
  output: string;
  operation: "video" | "audio" | "extract-audio" | "remux";
  videoCodec: string;
  audioCodec: string;
  width?: number | null;
  height?: number | null;
  fps?: number | null;
  hardwareDecode: boolean;
}


export interface AiChatResult {
  model: string;
  response: string;
  endpoint: string;
}
