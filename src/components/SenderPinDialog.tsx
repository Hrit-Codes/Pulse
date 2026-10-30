import { KeyRoundIcon, XIcon } from "lucide-react";

interface SenderPinDialogProps {
  pin: string;
  onClose: () => void;
}

export function SenderPinDialog({ pin, onClose }: SenderPinDialogProps) {
  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="sender-pin-title"
      onClick={onClose}
    >
      <div
        className="card max-w-sm w-full"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <KeyRoundIcon size={18} className="text-accent-light" />
            <h2 id="sender-pin-title" className="text-base font-semibold">
              Sender PIN
            </h2>
          </div>
          <button className="icon-btn icon-btn-ghost" onClick={onClose}>
            <XIcon size={18} />
          </button>
        </div>

        <p className="text-xs text-text-muted mb-4">
          Enter this PIN on the receiver to accept the transfer.
        </p>

        <div className="text-center text-3xl font-mono font-semibold tracking-[0.4em] text-accent-light py-2">
          {pin}
        </div>
      </div>
    </div>
  );
}
