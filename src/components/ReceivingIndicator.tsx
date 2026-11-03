import { DownloadIcon, LoaderIcon } from "lucide-react";

interface ReceivingIndicatorProps {
  filename: string;
  percent?: number | null;
  stacked?: boolean; // true when the sending indicator is also visible
}

export function ReceivingIndicator({ filename, percent, stacked }: ReceivingIndicatorProps) {
  const determinate = percent != null;
  return (
    <div
      className={`fixed right-4 z-40 card w-72 shadow-lg ${stacked ? "bottom-32" : "bottom-4"}`}
      role="status"
      aria-live="polite"
    >
      <div className="flex items-center gap-3">
        <DownloadIcon size={18} className="text-accent-light flex-shrink-0" />
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium">
            Receiving file...{determinate ? ` ${Math.round(percent!)}%` : ""}
          </p>
          <p className="text-xs text-text-muted truncate">{filename}</p>
        </div>
        <LoaderIcon size={16} className="animate-spin text-accent-light flex-shrink-0" />
      </div>

      <div className="mt-3 h-1.5 rounded-full bg-bg-card-hover overflow-hidden relative">
        {determinate ? (
          <div
            className="h-full rounded-full bg-accent-gradient transition-all duration-150"
            style={{ width: `${Math.min(percent!, 100)}%` }}
          />
        ) : (
          <div className="absolute inset-y-0 w-1/3 rounded-full bg-accent-gradient animate-[receiving_1.2s_ease-in-out_infinite]" />
        )}
      </div>
      <style>{`
        @keyframes receiving {
          0%   { left: -33%; }
          100% { left: 100%; }
        }
      `}</style>
    </div>
  );
}