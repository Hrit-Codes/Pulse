import { LoaderIcon, SendIcon } from "lucide-react";

interface SendingIndicatorProps {
  filename: string;
  percent?: number | null;
}

export function SendingIndicator({ filename, percent }: SendingIndicatorProps) {
  const determinate = percent != null;
  return (
    <div
      className="fixed bottom-4 right-4 z-40 card w-72 shadow-lg"
      role="status"
      aria-live="polite"
    >
      <div className="flex items-center gap-3">
        <SendIcon size={18} className="text-accent-light flex-shrink-0" />
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium">
            Sending file...{determinate ? ` ${Math.round(percent!)}%` : ""}
          </p>
          <p className="text-xs text-text-muted truncate">{filename}</p>
        </div>
        <LoaderIcon size={16} className="animate-spin text-accent-light flex-shrink-0" />
      </div>
    </div>
  );
}
