export type DeviceStatus = "online" | "offline";

export interface Device {
  id: string;
  name: string;
  ip: string;
  status: DeviceStatus;
  kind: "laptop" | "desktop";
}

export type TransferStatus =
  | "queued"
  | "connecting"
  | "transferring"
  | "paused"
  | "failed"
  | "completed"
  | "cancelled";

export interface Transfer {
  id: string;
  filename: string;
  sizeBytes: number;
  transferredBytes: number;
  direction: "sent" | "received";
  deviceName: string;
  status: TransferStatus;
  speedMBps: number;
  etaSeconds: number | null;
  chunkTotal: number;
  chunksDone: number;
  verified: boolean | null; // null until the final hash check runs
  hash: string;
  date: string;
}

export interface SyncFolder {
  id: string;
  path: string;
  deviceNames: string[];
  totalFiles: number;
  syncedFiles: number;
  conflictCount: number;
  pendingCount: number;
}

export interface SyncConflictVersion {
  deviceName: string;
  modifiedAt: string;
}

export interface SyncConflict {
  id: string;
  folderId: string;
  filename: string;
  versionA: SyncConflictVersion;
  versionB: SyncConflictVersion;
}

export interface PairingRequest {
  deviceName: string;
  code: string; // 6-digit code, shown on both ends
}

export type PageId = "devices" | "transfers" | "sync" | "history" | "settings";
