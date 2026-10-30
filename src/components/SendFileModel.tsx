import { useEffect, useState } from "react";
import { Device } from "../types";
import { UploadIcon, XIcon, FileIcon } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { baseName } from "../format";
import { getCurrentWebview } from "@tauri-apps/api/webview";

interface SendFileModalProps {
  device: Device;
  onClose: () => void;
  onSend: (path: string) => void;
}

export function SendFileModal({ device, onClose, onSend }: SendFileModalProps) {
  const [selectedPath, setSelectedPath] = useState<string>();
  const [isDragOver, setIsDragOver]=useState(false);

  // Native Tauri drag-drop (HTML5 drops don't give filesystem paths)
  useEffect(()=>{
    let unlisten:(()=>void) | undefined;
    let cancelled=false;

    getCurrentWebview()
      .onDragDropEvent((event)=>{
        const p=event.payload;
        if(p.type==="enter" || p.type==="over"){
          setIsDragOver(true);
        }else if(p.type==="leave"){
          setIsDragOver(false);
        }else if(p.type==="drop"){
          setIsDragOver(true);
          setSelectedPath(`${p.paths}`);
        }
      })
      .then((fn)=>{
        if(cancelled) fn();
        else unlisten= fn;
      })

      return()=>{
        cancelled=true;
        unlisten?.();
      }
  },[])

  const pickFiles = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: "Select files to send",
      });
      if (!selected) return;
      const path = Array.isArray(selected) ? selected[0] : selected;
      setSelectedPath(path);
    } catch (e) {
      console.log("Failed to pick files::", e);
    }
  };

  const handleRemoveFile = () => {
    setSelectedPath("");
  };

  const handleSend = () => {
    if (selectedPath) {
      onSend(selectedPath);
    }
  };

  return (
    <div
      className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4"
      onClick={onClose}
    >
      <div
        className="card max-w-md w-full max-h-[90vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 className="text-base font-semibold">Send to {device.name}</h2>
            <p className="text-xs text-text-muted font-mono">
              {device.ip}:{device.port}
            </p>
          </div>
          <button className="icon-btn icon-btn-ghost" onClick={onClose}>
            <XIcon size={18} />
          </button>
        </div>

        {/* Dropzone */}
        <div
          className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer ${
            isDragOver || selectedPath
              ? "border-accent-light bg-accent-dim"
              : "border-border-strong hover:border-accent-light hover:bg-accent-dim/50"
          }`}
          onClick={pickFiles}
        >
          <UploadIcon
            size={32}
            className={`mx-auto mb-2 ${
              selectedPath ? "text-accent-light" : "text-text-faint"
            }`}
          />
          <p className="text-sm font-medium">
            {selectedPath
              ? `${selectedPath} file selected`
              : "Click to browse files"}
          </p>
          {/* <p className="text-xs text-text-faint mt-1">
            {selectedPaths
              ? selectedPaths.map((p) => baseName(p)).join(", ")
              : "Any file type supported"}
          </p> */}
        </div>

        {/* Selected Files List */}
        {selectedPath && (
          <div className="mt-4 space-y-1.5">
              <div
                className="flex items-center gap-2 p-2 rounded bg-bg-card-hover border border-border"
              >
                <FileIcon size={14} className="text-text-muted flex-shrink-0" />
                <span className="text-sm truncate flex-1">{baseName(selectedPath)}</span>
                <button
                  className="icon-btn icon-btn-ghost text-text-faint hover:text-danger"
                  onClick={() => handleRemoveFile()}
                >
                  <XIcon size={14} />
                </button>
              </div>
          </div>
        )}

        {/* Actions */}
        <div className="flex items-center justify-end gap-2 mt-4 pt-4 border-t border-border">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            disabled={!selectedPath}
            onClick={handleSend}
          >
            Send 
          </button>
        </div>
      </div>
    </div>
  );
}
