import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Device } from "../types";
import {
  broadcast,
  getIdentity,
  listDevices,
  onDevicesChanged,
  type DeviceInfo,
} from "../api";

const DESKTOP_HINTS = [
  "desktop",
  "imac",
  "mac mini",
  "mac studio",
  "workstation",
  "server",
  " pc",
];

// how long we wait for peers to reply to a discovery broadcast
const SCAN_WINDOW_MS = 3200;

function inferKind(name: string): Device["kind"] {
  const lowered = name.toLowerCase();
  return DESKTOP_HINTS.some((hint) => lowered.includes(hint)) ? "desktop" : "laptop";
}

function toDevice(info: DeviceInfo, ip: string): Device {
  return {
    id: info.id,
    name: info.name,
    ip,
    port: info.port,
    status: "online",
    kind: inferKind(info.name),
  };
}

export function useDevices() {
  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const myIdRef = useRef<string | null>(null);

  const refresh = useCallback(async () => {
    const found = await listDevices();
    const unique = new Map<string, Device>();
    for (const entry of found ?? []) {
      if (!Array.isArray(entry) || entry.length < 2) continue;
      const [info, ip] = entry;
      if (!info?.id || !ip) continue;
      if (info.id === myIdRef.current) continue;
      unique.set(info.id, toDevice(info, ip));
    }
    setDevices([...unique.values()]);
  }, []);

  const scan = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await broadcast();
      // replies land on the always-on discovery listener; give them time to arrive
      await new Promise((resolve) => setTimeout(resolve, SCAN_WINDOW_MS));
      await refresh();
    } catch (e) {
      console.log("Failed to scan devices::", e);
      setDevices([]);
      setError("Could not scan the network.");
    } finally {
      setLoading(false);
    }
  }, [refresh]);

  useEffect(() => {
    let active = true;
    getIdentity()
      .then((me) => {
        if (active) myIdRef.current = me.id;
      })
      .catch(() => {});
    // peers that discover us (e.g. while they scan) show up without us rescanning
    const unlisten = onDevicesChanged(() => {
      refresh().catch(() => {});
    });
    scan();
    return () => {
      active = false;
      unlisten.then((unsub) => unsub());
    };
  }, [scan, refresh]);

  const onlineCount = useMemo(
    () => devices.filter((d) => d.status === "online").length,
    [devices]
  );

  return { devices, loading, error, onlineCount, scan };
}
