import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { Device } from "../types";

const DESKTOP_HINTS = [
  "desktop",
  "imac",
  "mac mini",
  "mac studio",
  "workstation",
  "server",
  " pc",
];

function inferKind(name: string): Device["kind"] {
  const lowered = name.toLowerCase();
  return DESKTOP_HINTS.some((hint) => lowered.includes(hint)) ? "desktop" : "laptop";
}

interface DiscoveredDeviceInfo {
  id: string;
  name: string;
  port: number;
}

type ListDevicesResult = Array<[DiscoveredDeviceInfo, string]>;

function toDevice(info: DiscoveredDeviceInfo, ip: string): Device {
  return {
    id: info.id,
    name: info.name,
    ip,
    status: "online",
    kind: inferKind(info.name),
  };
}

export function useDevices() {
  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const scan = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke("broadcast");
      const found = await invoke<ListDevicesResult>("list_devices");
      const unique = new Map<string, Device>();
      for (const entry of found ?? []) {
        if (!Array.isArray(entry) || entry.length < 2) continue;
        const [info, ip] = entry;
        if (!info?.id || !ip) continue;
        unique.set(info.id, toDevice(info, ip));
      }
      setDevices([...unique.values()]);
    } catch (e) {
      console.log("Failed to scan devices::", e);
      setDevices([]);
      setError("Could not scan the network. Check that discovery is enabled in Settings.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    scan();
  }, [scan]);

  const onlineCount = useMemo(
    () => devices.filter((d) => d.status === "online").length,
    [devices]
  );

  return { devices, loading, error, onlineCount, scan };
}
