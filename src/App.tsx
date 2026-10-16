import { useEffect, useMemo, useRef, useState } from "react";
import { Device, PageId, Transfer } from "./types";
import { mockDevices } from "./mockData";
import Sidebar from "./components/Sidebar";
import { DevicesPage } from "./components/DevicesPage";
import { SettingsPage } from "./components/SettingsPage";
import { PinPrompt } from "./components/PinPrompt";
import {
  DeviceInfo,
  getIdentity,
  getLocalIp,
  getPendingTransfers,
  onPinRequired,
  onSenderPin,
  onError,
  peerAddr,
  receiverAddr,
  resume,
  resumeAddr,
  sendFile,
  startReceiver,
  startResumeListener,
  stopReceiver,
  stopResumeListener,
  submitPin,
} from "./api";
import { baseName } from "./format";

const LOCAL_DEVICE_NAME = "User Device";

export default function App() {
  const [page, setPage] = useState<PageId>("devices");
  const [localIp, setLocalIp] = useState<string>("-");
  const [identity, setIdentity] = useState<DeviceInfo | null>(null);
  const [transfers, setTransfers] = useState<Transfer[]>([]);
  const [resumableIds, setResumableIds] = useState<Set<string>>(new Set());
  const [pinRequest, setPinRequest] = useState<string | null>(null);
  const [senderPin, setSenderPin] = useState<string | null>(null);
  const sendingRef = useRef(false);

  useEffect(() => {
    const load = async () => {
      try {
        const ip = await getLocalIp();
        setLocalIp(ip);
      } catch (err) {
        console.log("Failed to get local IP::", err);
      }
      try {
        const id = await getIdentity();
        setIdentity(id);
      } catch (err) {
        console.log("Failed to get identity::", err);
      }
      try {
        const pending = await getPendingTransfers();
        const resumed: Transfer[] = pending.map(
          ([transferId, senderId, filename, fileSize, totalChunks]) => ({
            id: transferId,
            filename,
            sizeBytes: fileSize,
            transferredBytes: 0,
            direction: "received",
            deviceName: senderId,
            status: "paused",
            speedMBps: 0,
            etaSeconds: null,
            chunkTotal: totalChunks,
            chunksDone: 0,
            verified: null,
            hash: "pending",
            date: new Date().toISOString(),
          })
        );
        if (resumed.length > 0) {
          setResumableIds(new Set(resumed.map((t) => t.id)));
          setTransfers((prev) => [...resumed, ...prev]);
        }
      } catch (err) {
        console.log("Failed to load pending transfers::", err);
      }
    };
    load();
  }, []);

  useEffect(() => {
    if (!identity) return;

    const start = async () => {
      try {
        await startReceiver(receiverAddr(identity.port));
      } catch (err) {
        console.log("Failed to start receiver::", err);
      }
      try {
        await startResumeListener(resumeAddr(identity.port));
      } catch (err) {
        console.log("Failed to start resume listener::", err);
      }
    };
    start();

    const unlisteners: Array<Promise<() => void>> = [];
    unlisteners.push(
      onPinRequired((connectionId) => setPinRequest(connectionId)),
      onSenderPin((pin) => setSenderPin(pin)),
      onError((message) => console.log("backend error::", message))
    );

    return () => {
      stopReceiver();
      stopResumeListener();
      void Promise.all(unlisteners).then((stops) =>
        stops.forEach((stop) => stop())
      );
    };
  }, [identity]);

  const activeTransferCount = useMemo(
    () =>
      transfers.filter(
        (t) => t.status === "transferring" || t.status === "connecting"
      ).length,
    [transfers]
  );

  async function handleSendFiles(device: Device, path: string) {
    if (!identity) {
      console.log("Cannot send: identity not loaded yet");
      return;
    }
    if (sendingRef.current) {
      console.log("A send is already in progress");
      return;
    }

    sendingRef.current = true;

    const localId = `local-${Date.now()}-${Math.random()
      .toString(36)
      .slice(2)}`;

    const optimistic: Transfer = {
      id: localId,
      filename: baseName(path),
      sizeBytes: 0,
      transferredBytes: 0,
      direction: "sent",
      deviceName: device.name,
      status: "connecting",
      speedMBps: 0,
      etaSeconds: null,
      chunkTotal: 0,
      chunksDone: 0,
      verified: null,
      hash: "pending",
      date: new Date().toISOString(),
    };

    setTransfers((prev) => [optimistic, ...prev]);

    try {
      const addr = peerAddr(device.ip, device.port);
      await sendFile(identity.id, addr, path);
      setTransfers((prev) =>
        prev.map((t) =>
          t.id === localId
            ? { ...t, status: "completed", verified: true }
            : t
        )
      );
    } catch (err) {
      console.log("send_file failed::", err);
      setTransfers((prev) =>
        prev.map((t) =>
          t.id === localId ? { ...t, status: "failed" } : t
        )
      );
    } finally {
      sendingRef.current = false;
      setSenderPin(null);
    }
  }

  async function handlePinSubmit(pin: string) {
    if (!pinRequest) return;
    try {
      await submitPin(pinRequest, pin);
    } catch (err) {
      console.log("submit_pin failed::", err);
    } finally {
      setPinRequest(null);
    }
  }

  async function handleResume(transferId: string) {
    try {
      await resume(transferId);
      setTransfers((prev) =>
        prev.map((t) =>
          t.id === transferId ? { ...t, status: "transferring" } : t
        )
      );
      setResumableIds((prev) => {
        const next = new Set(prev);
        next.delete(transferId);
        return next;
      });
    } catch (err) {
      console.log("resume failed::", err);
    }
  }

  return (
    <div className="app-shell">
      <Sidebar
        active={page}
        onNavigate={setPage}
        localDeviceName={identity?.name ?? LOCAL_DEVICE_NAME}
        localIp={localIp}
        activeTransferCount={activeTransferCount}
      />
      <main className="app-main">
        {page === "devices" && (
          <DevicesPage
            recentTransfers={transfers.slice(0, 4)}
            onSendFiles={handleSendFiles}
            resumableIds={resumableIds}
            onResume={handleResume}
          />
        )}
        {page === "settings" && (
          <SettingsPage
            localDeviceName={identity?.name ?? LOCAL_DEVICE_NAME}
            localIp={localIp}
            trustedDevices={mockDevices}
          />
        )}
      </main>

      {senderPin && (
        <div className="fixed bottom-6 right-6 z-40 card px-4 py-3 flex flex-col gap-1">
          <span className="text-xs text-text-muted">
            PIN to enter on receiver
          </span>
          <span className="text-2xl font-mono font-semibold tracking-[0.4em] text-accent-light">
            {senderPin}
          </span>
        </div>
      )}

      {pinRequest && (
        <PinPrompt
          onSubmit={handlePinSubmit}
          onCancel={() => setPinRequest(null)}
        />
      )}
    </div>
  );
}