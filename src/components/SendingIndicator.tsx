import { LoaderIcon, SendIcon } from "lucide-react";

interface SendingIndicatorProps {
  filename: string;
}

export function SendingIndicator({ filename }: SendingIndicatorProps) {
  return (
    <div
      className="fixed bottom-4 right-4 z-40 card w-72 shadow-lg"
      role="status"
      aria-live="polite"
    >
      <div className="flex items-center gap-3">
        <div className="relative flex-shrink-0">
          <SendIcon size={18} className="text-accent-light" />
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium">Sending file...</p>
          <p className="text-xs text-text-muted truncate">{filename}</p>
        </div>
        <LoaderIcon size={16} className="animate-spin text-accent-light flex-shrink-0" />
      </div>

      {/* indeterminate progress bar */}
      <div className="mt-3 h-1.5 rounded-full bg-bg-card-hover overflow-hidden relative">
        <div className="absolute inset-y-0 w-1/3 rounded-full bg-accent-gradient animate-[sending_1.2s_ease-in-out_infinite]" />
      </div>
      <style>{`
        @keyframes sending {
          0%   { left: -33%; }
          100% { left: 100%; }
        }
      `}</style>
    </div>
  );
}