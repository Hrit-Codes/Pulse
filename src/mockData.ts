import type { Device, SyncConflict, SyncFolder, Transfer } from "./types";

export const mockDevices: Device[] = [
  { id: "dev-1", name: "Hrit's MacBook", ip: "192.168.1.12", status: "online", kind: "laptop" },
  { id: "dev-2", name: "Alex's Laptop", ip: "192.168.1.21", status: "online", kind: "laptop" },
  { id: "dev-3", name: "Lab PC", ip: "192.168.1.34", status: "offline", kind: "desktop" },
];

export const mockTransfers: Transfer[] = [
  {
    id: "tx-1",
    filename: "project_backup.zip",
    sizeBytes: 5_368_709_120,
    transferredBytes: 4_781_684_736,
    direction: "sent",
    deviceName: "Alex's Laptop",
    status: "transferring",
    speedMBps: 92,
    etaSeconds: 6,
    chunkTotal: 5120,
    chunksDone: 4561,
    verified: null,
    hash: "a8d392f1",
    date: "2026-09-02T10:14:00Z",
  },
  {
    id: "tx-2",
    filename: "architecture.png",
    sizeBytes: 8_400_000,
    transferredBytes: 8_400_000,
    direction: "sent",
    deviceName: "Hrit's MacBook",
    status: "completed",
    speedMBps: 0,
    etaSeconds: 0,
    chunkTotal: 8,
    chunksDone: 8,
    verified: true,
    hash: "7c1e44b9",
    date: "2026-09-02T09:58:00Z",
  },
  {
    id: "tx-3",
    filename: "backup.tar",
    sizeBytes: 7_730_941_133,
    transferredBytes: 2_684_354_560,
    direction: "received",
    deviceName: "Lab PC",
    status: "paused",
    speedMBps: 0,
    etaSeconds: null,
    chunkTotal: 7373,
    chunksDone: 2560,
    verified: null,
    hash: "f0a2c8de",
    date: "2026-09-01T21:03:00Z",
  },
];

export const mockSyncFolders: SyncFolder[] = [
  {
    id: "folder-1",
    path: "~/Projects/Shared",
    deviceNames: ["Hrit's MacBook", "Alex's Laptop"],
    totalFiles: 342,
    syncedFiles: 339,
    conflictCount: 2,
    pendingCount: 1,
  },
];

export const mockConflicts: SyncConflict[] = [
  {
    id: "conflict-1",
    folderId: "folder-1",
    filename: "notes.md",
    versionA: { deviceName: "Hrit's MacBook", modifiedAt: "12:42 PM" },
    versionB: { deviceName: "Alex's Laptop", modifiedAt: "12:44 PM" },
  },
  {
    id: "conflict-2",
    folderId: "folder-1",
    filename: "README.md",
    versionA: { deviceName: "Hrit's MacBook", modifiedAt: "9:10 AM" },
    versionB: { deviceName: "Alex's Laptop", modifiedAt: "9:11 AM" },
  },
];
