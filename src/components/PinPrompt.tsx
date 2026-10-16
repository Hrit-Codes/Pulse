import { useState } from "react";
import { KeyRoundIcon, XIcon } from "lucide-react";

interface PinPromptProps {
  onSubmit: (pin: string) => void;
  onCancel: () => void;
}

export function PinPrompt({ onSubmit, onCancel }: PinPromptProps) {
  const [pin, setPin] = useState("");

  const submit = () => {
    if (pin.trim().length > 0) onSubmit(pin.trim());
  };

  return (
    <div
      className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4"
      onClick={onCancel}
    >
      <div className="card max-w-sm w-full" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <KeyRoundIcon size={18} className="text-accent-light" />
            <h2 className="text-base font-semibold">Enter PIN</h2>
          </div>
          <button className="icon-btn icon-btn-ghost" onClick={onCancel}>
            <XIcon size={18} />
          </button>
        </div>

        <p className="text-xs text-text-muted mb-3">
          A device wants to send you a file. Enter the 4-digit PIN shown on the
          sender's screen to accept the transfer.
        </p>

        <input
          autoFocus
          type="text"
          inputMode="numeric"
          maxLength={4}
          value={pin}
          onChange={(e) => setPin(e.target.value.replace(/\D/g, ""))}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          placeholder="0000"
          className="input w-full text-center text-2xl font-mono tracking-[0.5em]"
        />

        <div className="flex justify-end gap-2 mt-4 pt-4 border-t border-border">
          <button className="btn btn-secondary" onClick={onCancel}>
            Decline
          </button>
          <button
            className="btn btn-primary"
            disabled={pin.trim().length === 0}
            onClick={submit}
          >
            Accept
          </button>
        </div>
      </div>
    </div>
  );
}
