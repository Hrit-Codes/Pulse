import { LaptopIcon, MonitorIcon } from "lucide-react";
import { Device } from "../types";

interface DeviceCardProps {
  device: Device;
  onSend: (device: Device) => void;
}

export function DeviceCard({ device, onSend }: DeviceCardProps) {
  const isOnline = device.status === "online";

  return (
    <div className={`card p-4 flex flex-col gap-3 ${!isOnline ? "opacity-50" : ""}`}>
      <div className="flex items-center gap-3">
        {/* Device Icon */}
        <div className="p-2 rounded-lg bg-bg-card-hover border border-border text-text-muted">
          {device.kind === "desktop" ? <MonitorIcon size={18} /> : <LaptopIcon size={18} />}
        </div>

        {/* Device Info */}
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium truncate">{device.name}</p>
          <p className="text-xs text-text-muted font-mono">{device.ip}</p>
        </div>

        {/* Status Dot */}
        <div
          className={`w-2 h-2 rounded-full flex-shrink-0 ${
            isOnline ? "bg-success" : "bg-text-faint"
          } ${isOnline ? "animate-pulse" : ""}`}
        />
      </div>

      {/* Action Button */}
      <button
        className="btn btn-primary w-full justify-center text-sm"
        disabled={!isOnline}
        onClick={() => onSend(device)}
      >
        Send File
      </button>
    </div>
  );
}