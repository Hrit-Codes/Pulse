import { useMemo, useState } from "react";
import { Device, PageId, Transfer } from "./types";
import { mockDevices, mockTransfers } from "./mockData";
import Sidebar from "./components/Sidebar";
import { DevicesPage } from "./components/DevicesPage";
import { SettingsPage } from "./components/SettingsPage";

const LOCAL_DEVICE_NAME="Hrit's Macbook";
const LOCAL_IP="192.168.1.12";

export default function App(){
  const [page, setPage]=useState<PageId>("devices");
  const [transfers, setTransfers]=useState<Transfer[]>(mockTransfers);

  const activeTransferCount=useMemo(
    ()=>transfers.filter((t)=>t.status==="transferring"||t.status==="connecting").length,
    [transfers]
  )

  function handleSendFiles(device: Device, files: File[]) {
    // Wire this to: invoke("start_transfer", { deviceId: device.id, paths: [...] })
    // The Rust side should then emit "transfer://progress" events with the
    // shape of `Transfer` in types.ts, which you'd merge into state here.
    const newTransfer: Transfer = {
      id: `tx-${Date.now()}`,
      filename: files[0]?.name ?? "file",
      sizeBytes: files.reduce((sum, f) => sum + f.size, 0),
      transferredBytes: 0,
      direction: "sent",
      deviceName: device.name,
      status: "connecting",
      speedMBps: 0,
      etaSeconds: null,
      chunkTotal: 100,
      chunksDone: 0,
      verified: null,
      hash: "pending",
      date: new Date().toISOString(),
    };
    setTransfers((prev) => [newTransfer, ...prev]);
    setPage("transfers");
  }

  return(
    <div className="app-shell">
      <Sidebar
        active={page}
        onNavigate={setPage}
        localDeviceName={LOCAL_DEVICE_NAME}
        localIp={LOCAL_IP}
        activeTransferCount={activeTransferCount}
        />
        <main className="app-main">
          {page==="devices" && (
            <DevicesPage
              devices={mockDevices}
              recentTransfers={transfers.slice(0,4)}
              onSendFiles={handleSendFiles}
            />
          )}
          {page==="settings" && (
            <SettingsPage 
              localDeviceName={LOCAL_DEVICE_NAME}
              localIp={LOCAL_IP}
              trustedDevices={mockDevices}/>
          )}

        </main>

    </div>
  )
}