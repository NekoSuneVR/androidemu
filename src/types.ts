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
