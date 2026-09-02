import { useEffect, useState } from "react";
import type { Device } from "../types";
import { LockIcon, ShieldIcon, CheckCircleIcon, FolderIcon, SaveIcon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "../context/ThemeContext";
import { Store } from "@tauri-apps/plugin-store";
import { open } from '@tauri-apps/plugin-dialog';
import { downloadDir } from "@tauri-apps/api/path";

interface SettingsPageProps {
  localDeviceName: string;
  localIp: string;
  trustedDevices: Device[];
}

export function SettingsPage({ localDeviceName, localIp, trustedDevices }: SettingsPageProps) {
//   const [pairingRequest, setPairingRequest] = useState<PairingRequest | null>(null);
  const [isDiscoveryEnabled, setIsDiscoveryEnabled]=useState(true);
  const [isThemeDropdownOpen, setIsThemeDropdownOpen]=useState(false);
  const {theme, setTheme}=useTheme();
  const [downloadPath, setDownloadPath]=useState("");
  const [isLoading, setIsLoading]=useState(true);
  const [isSaving, setIsSaving]=useState(false);

  useEffect(()=>{
    const loadDiscoveryState=async()=>{
        try{
            const state=await invoke<boolean>("get_discovery_state");
            setIsDiscoveryEnabled(state);
        }catch(error){
            console.log("Failed to load discovery state:",error);
        }
    }
    loadDiscoveryState();
  },[]);

  const toggleDisovery=async()=>{
    const newState=!isDiscoveryEnabled;
    try{
        await invoke ("set_discovery_state",{enabled:newState});
        setIsDiscoveryEnabled(newState);
    }catch(error){
        console.log("Failed to toggle discovery",error);
    }
  }

  const themeOptions=[
    { value:"system", label:"System" },
    { value:"light", label:"Light" },
    { value:"dark", label:"Dark" }
  ]

  const currentTheme = themeOptions.find((t) => t.value === theme);

  useEffect(()=>{
    const load=async()=>{
        try{
            const store=await Store.load("settings.json");
            const stored=await store.get<string>("download_path");
            console.log("Stored download_path:",stored);
            setDownloadPath(stored || "~/Donwloads/Pulse");
        }catch(e){
            console.log(e);
        }finally{
            setIsLoading(false);
        }
    }
    load();
  },[]);

  const savePath=async()=>{
    setIsSaving(true);
    try{
        const store=await Store.load("settings.json");
        await store.set("download_path",downloadPath)
        await store.save();
    }catch(e){
        console.log(e);
    }finally{{
        setIsSaving(false);
    }}
  }

  const browseFolder=async()=>{
    try{
        const fallback = await downloadDir();
        const selected=await open({
            directory:true,
            multiple:false,
            title:"Select Download Folder",
            defaultPath:downloadPath || fallback,
        });
        if(selected){
            setDownloadPath(selected as string);
        }
    }catch(e){
        console.log("Browser error::",e);
    }
  }


  return (
    <div className="page">
      {/* Page Header */}
      <header className="page-header">
        <div>
          <h1 className="page-title">Settings</h1>
          <p className="page-subtitle">Device identity, trusted peers, and security</p>
        </div>
      </header>

      <section className="card mb-4">
        <h2 className="card-title">This device</h2>
        <div className="space-y-3">
          <div className="flex items-center justify-between py-2 border-b border-border">
            <span className="text-sm text-text-muted">DEVICE NAME</span>
            <span className="text-sm font-medium">{localDeviceName}</span>
          </div>
          <div className="flex items-center justify-between py-2 border-b border-border">
            <span className="text-sm text-text-muted">LOCAL IP</span>
            <span className="text-sm font-mono text-text">{localIp}</span>
          </div>
          <div className="flex items-center justify-between py-2 border-b border-border">
            <span className="text-sm text-text-muted">Discovery status</span>
            <span className="flex items-center gap-2">
              <span className="status-dot inline-block" />
              <span className="text-sm text-success">Online</span>
              <button
                type="button"
                onClick={toggleDisovery}
                className={`relative w-10 h-5 rounded-full transition-colors ${isDiscoveryEnabled? "bg-success":"bg-danger-dim"}`}
                role="switch"
                aria-checked={isDiscoveryEnabled}
                >
                    <span className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform ${isDiscoveryEnabled?"translate-x-5":"translate-x-0"}`}>

                    </span>
                </button>
            </span>
          </div>
          <div className="flex items-center justify-between py-2 border-b border-border">
            <span className="text-sm text-text-muted">Theme status</span>
            <div className="relative">
                <button
                    type="button"
                    onClick={()=>setIsThemeDropdownOpen(!isThemeDropdownOpen)}
                    className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-bg-card-hover border border-border hover:border-border-strong transition-colors min-w-[120px]"
                >
                    {currentTheme && (
                        <span className="text-sm flex-1 text-center">{currentTheme.label}</span>
                    )}
                </button>
                {isThemeDropdownOpen && (
                    <div className="absolute right-0 top-full mt-1 w-full min-w-[140px] bg-bg-card border-border rounded-md shad0w-lg overflow-hidden z-10">
                        {themeOptions.map((option)=>{
                            const isActive=theme===option.value
                            return(
                                <button
                                    key={option.value}
                                    type="button"
                                    onClick={()=>{
                                        setTheme(option.value as "system"|"light"|"dark");
                                        setIsThemeDropdownOpen(false);
                                    }}
                                    className={`w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors ${
                                        isActive
                                        ? "bg-accent-gradient text-text"
                                        : "text-text hover:bg-bg-card-hover hover:cursor-pointer"
                                    }`}
                                >
                                    {option.label}
                                </button>
                            )
                        })}

                    </div>
                )}
            </div>
          </div>
        </div>
      </section>

      <section className="card mb-4">
        <h2 className="card-title">Download Settings</h2>
        <div className="flex items-center gap-2 py-2">
            <input
                type="text"
                value={downloadPath}
                onChange={(e)=>setDownloadPath(e.target.value)}
                className="input flex-1"
                disabled={isLoading}
            />
            <button className="btn btn-secondary px-2 py-1" onClick={browseFolder}>
                <FolderIcon size={16}/>
            </button>
            <button
                className="btn btn-primary px-2 py-1"
                onClick={savePath}
                disabled={isSaving}
            >
                <SaveIcon size={16}/>
            </button>

        </div>

      </section>

      <section className="card mb-4">
        <div className="card-header">
          <h2 className="card-title">Trusted devices</h2>
          <span className="badge badge-accent">{trustedDevices.length} paired</span>
        </div>

        {trustedDevices.length > 0 ? (
          <div className="space-y-2">
            {trustedDevices.map((device) => (
              <div
                key={device.id}
                className="flex items-center justify-between p-2.5 rounded bg-bg-card-hover border border-border"
              >
                <div>
                  <p className="text-sm font-medium">{device.name}</p>
                  <p className="text-xs text-text-muted font-mono">{device.ip}</p>
                </div>
                <span className="w-fit badge badge-success flex flex-row items-center gap-1">
                  <LockIcon size={12} /> PAIRED
                </span>
              </div>
            ))}
          </div>
        ) : (
          <div className="empty-state text-sm">
            <p>No trusted devices yet</p>
            <p className="text-xs text-text-faint mt-1">
              Pair with devices to enable secure transfers
            </p>
          </div>
        )}
      </section>

      <section className="card">
        <h2 className="card-title">Security</h2>
        <div className="space-y-3">
          <div className="flex items-center justify-between py-2 border-b border-border">
            <span className="text-sm text-text-muted">Transport encryption</span>
            <span className="flex items-center gap-1.5 text-sm text-success">
              <CheckCircleIcon size={14} />
              TLS enabled
            </span>
          </div>
          <div className="flex items-center justify-between py-2">
            <span className="text-sm text-text-muted">Integrity check</span>
            <span className="flex items-center gap-1.5 text-sm text-text">
              <ShieldIcon size={14} className="text-accent-light" />
              SHA-256 on every completed transfer
            </span>
          </div>
        </div>

        {/* Simulate Pairing Button */}
        {/* <button
          type="button"
          className="btn btn-secondary w-full justify-center mt-4 text-sm"
          onClick={() =>
            setPairingRequest({
              deviceName: "Unknown device (192.168.1.44)",
              code: "742 183",
            })
          }
        >
          Simulate incoming pairing request
        </button> */}
      </section>
    </div>
  );
}