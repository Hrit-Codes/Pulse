import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface DeviceInfo {
  id: string;
  name: string;
  port: number;
}

export type DiscoveredDevice = [DeviceInfo, string];

export type PendingTransfer = [string, string, string, number, number];

export function receiverAddr(port: number): string {
  return `0.0.0.0:${port}`;
}

export function resumeAddr(port: number): string {
  return `0.0.0.0:${port + 1}`;
}

export function peerAddr(ip: string, port: number): string {
  return `${ip}:${port}`;
}

function toError(err: unknown, fallback: string): Error {
  if (err instanceof Error) return err;
  if (typeof err === "string") return new Error(err);
  return new Error(fallback);
}

export async function getIdentity(): Promise<DeviceInfo> {
  try {
    return await invoke<DeviceInfo>("get_identity");
  } catch (err) {
    throw toError(err, "Failed to load device identity");
  }
}

export async function getLocalIp(): Promise<string> {
  try {
    return await invoke<string>("get_local_ip");
  } catch (err) {
    throw toError(err, "Failed to resolve local IP");
  }
}

export async function startReceiver(addr: string): Promise<void> {
  try {
    await invoke("start_receiver", { addr });
  } catch (err) {
    throw toError(err, `Failed to start receiver on ${addr}`);
  }
}

export async function stopReceiver(): Promise<void> {
  await invoke("stop_receiver");
}

export async function broadcast(): Promise<void> {
  try {
    await invoke("broadcast");
  } catch (err) {
    throw toError(err, "Failed to broadcast discovery");
  }
}

export async function listDevices(): Promise<DiscoveredDevice[]> {
  try {
    return await invoke<DiscoveredDevice[]>("list_devices");
  } catch (err) {
    throw toError(err, "Failed to list devices");
  }
}

export async function sendFile(
  senderId: string,
  addr: string,
  filePath: string
): Promise<void> {
  try {
    await invoke("send_file", { senderId, addr, filePath });
  } catch (err) {
    throw toError(err, `Failed to send file to ${addr}`);
  }
}

export async function startResumeListener(addr: string): Promise<void> {
  try {
    await invoke("start_resume_listener", { addr });
  } catch (err) {
    throw toError(err, `Failed to start resume listener on ${addr}`);
  }
}

export async function stopResumeListener(): Promise<void> {
  await invoke("stop_resume_listener");
}

export async function resume(transferId: string): Promise<void> {
  try {
    await invoke("resume", { transferId });
  } catch (err) {
    throw toError(err, `Failed to resume transfer ${transferId}`);
  }
}

export async function submitPin(
  connectionId: string,
  pin: string
): Promise<void> {
  try {
    await invoke("submit_pin", { connectionId, pin });
  } catch (err) {
    throw toError(err, "Failed to submit PIN");
  }
}

export async function getPendingTransfers(): Promise<PendingTransfer[]> {
  try {
    return await invoke<PendingTransfer[]>("get_pending_transfers");
  } catch (err) {
    throw toError(err, "Failed to load pending transfers");
  }
}

export function onPinRequired(
  handler: (connectionId: string) => void
): Promise<UnlistenFn> {
  return listen<string>("pin_required", (event) => handler(event.payload));
}

export function onSenderPin(handler: (pin: string) => void): Promise<UnlistenFn> {
  return listen<string>("pin", (event) => handler(event.payload));
}

export function onTransferComplete(
  handler: (message: string) => void
): Promise<UnlistenFn> {
  return listen<string>("transfer_complete_sender", (event) =>
    handler(event.payload)
  );
}

export function onError(handler: (message: string) => void): Promise<UnlistenFn> {
  return listen<string>("error", (event) => handler(event.payload));
}

export function onDevicesChanged(handler: () => void): Promise<UnlistenFn> {
  return listen("devices_changed", () => handler());
}
