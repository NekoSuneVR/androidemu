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
  storageGb: number;
  wifi: boolean;
  bluetooth: boolean;
  gps: boolean;
  cameraConfiguration: "none" | "front" | "rear" | "front+rear";
  microphone: boolean;
  accelerometer: boolean;
  gyroscope: boolean;
  compass: boolean;
  lightSensor: boolean;
  proximitySensor: boolean;
  batteryPercent: number;
  charging: boolean;
  tabletResources: boolean;
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
  gmsProvider: "none" | "google-compatible" | "microg" | "custom-gapps";
  certificationStatus: string;
  playStorePackage?: string | null;
  secureImage: boolean;
  verifiedBootState: string;
  securityState: string;
  missingHardwareFeatures: string[];
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
  controlMode: "manual" | "assistant" | "accessibility" | "full-automation";
  helperMode: "inventory" | "quest" | "ui" | "repetitive-task";
  enforcePackageAllowlist: boolean;
  allowedPackages: string[];
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
  codecCapabilities: string[];
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
  openglAvailable: boolean;
  openglDetail: string;
  directxAvailable: boolean;
  directxDetail: string;
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
  operation: "video" | "compress" | "audio" | "extract-audio" | "remux";
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

export interface UpdateCheck {
  currentVersion: string;
  latestVersion: string;
  updateAvailable: boolean;
  releaseUrl: string;
}

export interface SkillControl {
  id: string;
  label: string;
  action: string;
  x?: number | null;
  y?: number | null;
  keycode?: string | null;
}
export interface SkillUiRegion {
  id: string;
  label: string;
  x: number;
  y: number;
  width: number;
  height: number;
}
export interface SkillManifest {
  schemaVersion: number;
  id: string;
  name: string;
  version: string;
  packages: string[];
  orientations: string[];
  controls: SkillControl[];
  uiRegions: SkillUiRegion[];
  systemPrompt: string;
  repositoryUrl?: string | null;
}
export interface AiGameState {
  packageName: string;
  activity: string;
  orientation: string;
  displaySize: string;
}

export interface ApkCompatibility {
  path: string;
  abis: string[];
  preferredAbi: string;
  nativeX86_64: boolean;
  needsArmCompatibility: boolean;
  diagnostic: string;
}

export interface KeyBinding {
  input: string;
  action: "tap"|"hold"|"swipe"|"key"|"text"|"mouse-left"|"mouse-right"|"scroll-up"|"scroll-down"|"joystick-up"|"joystick-down"|"joystick-left"|"joystick-right";
  x?: number|null; y?: number|null; x2?: number|null; y2?: number|null;
  durationMs?: number|null; keycode?: string|null; text?: string|null;
}
export interface KeymapProfile {
  schemaVersion: number;
  id: string;
  name: string;
  packageName: string;
  overlayVisible: boolean;
  bindings: KeyBinding[];
}

export interface GameSettings {
  packageName: string;
  orientation?: "portrait"|"landscape"|"reverse-portrait"|"reverse-landscape"|"automatic"|null;
  dpi?: number|null;
  fps?: number|null;
  keymapId?: string|null;
  aiSkillId?: string|null;
  notes: string;
}
