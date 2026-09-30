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
}

export interface HostCapabilities {
  os: string;
  arch: string;
  accelerator: string;
  virtualizationNote: string;
}
