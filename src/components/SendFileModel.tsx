import React, { useState, useRef } from "react";
import { Device } from "../types";
import { UploadIcon, XIcon, FileIcon } from "lucide-react";

interface SendFileModalProps {
  device: Device;
  onClose: () => void;
  onSend: (files: File[]) => void;
}

export function SendFileModal({ device, onClose, onSend }: SendFileModalProps) {
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      setSelectedFiles(Array.from(e.target.files));
    }
  };

  const handleDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    if (e.dataTransfer.files) {
      setSelectedFiles(Array.from(e.dataTransfer.files));
    }
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
  };

  const handleRemoveFile = (index: number) => {
    setSelectedFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSend = () => {
    if (selectedFiles.length > 0) {
      onSend(selectedFiles);
    }
  };

  const handleClick = () => {
    fileInputRef.current?.click();
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
            <p className="text-xs text-text-muted font-mono">{device.ip}</p>
          </div>
          <button
            className="icon-btn icon-btn-ghost"
            onClick={onClose}
          >
            <XIcon size={18} />
          </button>
        </div>

        {/* Dropzone */}
        <div
          className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer ${
            selectedFiles.length > 0
              ? "border-accent-light bg-accent-dim"
              : "border-border-strong hover:border-accent-light hover:bg-accent-dim/50"
          }`}
          onClick={handleClick}
          onDrop={handleDrop}
          onDragOver={handleDragOver}
        >
          <UploadIcon
            size={32}
            className={`mx-auto mb-2 ${
              selectedFiles.length > 0 ? "text-accent-light" : "text-text-faint"
            }`}
          />
          <p className="text-sm font-medium">
            {selectedFiles.length > 0
              ? `${selectedFiles.length} file(s) selected`
              : "Click to browse or drag files here"}
          </p>
          <p className="text-xs text-text-faint mt-1">
            {selectedFiles.length > 0
              ? selectedFiles.map((f) => f.name).join(", ")
              : "Any file type supported"}
          </p>
          <input
            ref={fileInputRef}
            type="file"
            multiple
            className="hidden"
            onChange={handleFileChange}
          />
        </div>

        {/* Selected Files List */}
        {selectedFiles.length > 0 && (
          <div className="mt-4 space-y-1.5">
            {selectedFiles.map((file, index) => (
              <div
                key={index}
                className="flex items-center gap-2 p-2 rounded bg-bg-card-hover border border-border"
              >
                <FileIcon size={14} className="text-text-muted flex-shrink-0" />
                <span className="text-sm truncate flex-1">{file.name}</span>
                <span className="text-xs text-text-faint flex-shrink-0">
                  {(file.size / 1024).toFixed(1)} KB
                </span>
                <button
                  className="icon-btn icon-btn-ghost text-text-faint hover:text-danger"
                  onClick={() => handleRemoveFile(index)}
                >
                  <XIcon size={14} />
                </button>
              </div>
            ))}
          </div>
        )}

        {/* Actions */}
        <div className="flex items-center justify-end gap-2 mt-4 pt-4 border-t border-border">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            disabled={selectedFiles.length === 0}
            onClick={handleSend}
          >
            Send {selectedFiles.length > 0 ? `(${selectedFiles.length})` : ""}
          </button>
        </div>
      </div>
    </div>
  );
}