import type { LucideIcon } from "lucide-react";
import { ClockIcon, FolderIcon, LaptopIcon, Settings, WifiIcon } from "lucide-react";
import type { PageId } from "../types";

interface NavItem {
  id: PageId;
  label: string;
  icon: LucideIcon;
}

const NAV_ITEMS: NavItem[] = [
  { id: "devices", label: "Devices", icon: WifiIcon },
  { id: "transfers", label: "Transfers", icon: LaptopIcon },
  { id: "sync", label: "Sync", icon: FolderIcon },
  { id: "history", label: "History", icon: ClockIcon },
  { id: "settings", label: "Settings", icon: Settings },
];

type SidebarProps = {
  active: PageId;
  onNavigate: (page: PageId) => void;
  localDeviceName: string;
  localIp: string;
  activeTransferCount: number;
};

export default function Sidebar({
  active,
  onNavigate,
  localDeviceName,
  localIp,
  activeTransferCount,
}: SidebarProps) {
  return (
    <aside className="w-[216px] shrink-0 flex flex-col border-r border-border bg-bg-sidebar px-3.5 py-5">
      {/* Brand */}
      <div className="flex items-center gap-2.5 px-1.5 pb-5">
        <img
          src="/Logo.png"
          alt="Pulse"
          className="w-8 h-8 rounded-md object-cover object-center"
        />
        <span className="font-bold text-[15px] tracking-tight text-accent-strong">
          Pulse
        </span>
      </div>

      {/* Local device status */}
      <div className="flex items-center gap-2.5 px-2.5 py-2.5 mb-4 bg-bg-card border border-border rounded-lg">
        <span className="relative flex h-2 w-2 shrink-0">
          <span className="absolute inline-flex h-full w-full rounded-full bg-success opacity-75 animate-ping" />
          <span className="relative inline-flex h-2 w-2 rounded-full bg-success" />
        </span>
        <div className="flex flex-col min-w-0">
          <span className="text-sm font-medium text-text truncate">
            {localDeviceName}
          </span>
          <span className="font-mono text-xs text-text-muted">{localIp}</span>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex flex-col gap-0.5">
        {NAV_ITEMS.map((item) => {
          const Icon = item.icon;
          const isActive = item.id === active;
          const badge =
            item.id === "transfers" && activeTransferCount > 0
              ? activeTransferCount
              : null;

          return (
            <button
              key={item.id}
              type="button"
              onClick={() => onNavigate(item.id)}
              aria-current={isActive ? "page" : undefined}
              className={`group w-full flex items-center gap-2.5 px-2.5 py-2 rounded text-sm text-left cursor-pointer transition-colors ${
                isActive
                  ? "bg-accent-dim text-accent-light"
                  : "text-text-muted hover:bg-bg-card-hover hover:text-text"
              }`}
            >
              <Icon
                size={17}
                className={isActive ? "text-accent-light" : "text-text-muted group-hover:text-text"}
              />
              <span className="flex-1">{item.label}</span>
              {badge !== null && (
                <span className="min-w-[18px] h-4 px-1 flex items-center justify-center rounded-full bg-accent-gradient text-[10.5px] font-bold text-[#fdf1f0]">
                  {badge}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="mt-auto pt-2.5 px-2.5 border-t border-border">
        <span className="text-xs text-text-faint">LAN-only · no cloud relay</span>
      </div>
    </aside>
  );
}