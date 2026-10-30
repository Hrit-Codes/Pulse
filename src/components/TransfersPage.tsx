import type { ReactNode } from "react";
import {
  ArrowLeftRightIcon,
  CheckIcon,
  FileWarningIcon,
  LoaderIcon,
  PauseIcon,
  PlayIcon,
} from "lucide-react";
import type { Transfer } from "../types";
import { formatBytes } from "../format";

type TransfersPageProps = {
  allTransfers: Transfer[];
  resumeableIds: Set<string>;
  onResume: (transferId: string) => void;
};

const STATUS_META: Record<string, { icon: ReactNode; className: string; label: string }> = {
  completed: { icon: <CheckIcon size={14} />, className: "text-success", label: "Complete" },
  paused: { icon: <PauseIcon size={14} />, className: "text-warning", label: "Paused" },
  failed: { icon: <FileWarningIcon size={14} />, className: "text-danger", label: "Failed" },
  transferring: {
    icon: <div className="w-2 h-2 rounded-full bg-accent-light animate-pulse" />,
    className: "text-accent-light",
    label: "In progress",
  },
  connecting: {
    icon: <LoaderIcon size={14} className="animate-spin" />,
    className: "text-accent-light",
    label: "Connecting",
  },
};

export function TransfersPage({
  allTransfers,
  resumeableIds,
  onResume,
}: TransfersPageProps) {
  const completeCount = allTransfers.filter((t) => t.status === "completed").length;

  return (
    <div className="page">
      {/* Page Header */}
      <header className="page-header">
        <div>
          <h1 className="page-title">Transfers</h1>
          <p className="page-subtitle">
            {allTransfers.length === 0
              ? "Your sent and received files will appear here"
              : `${allTransfers.length} transfer${allTransfers.length === 1 ? "" : "s"} in this session`}
          </p>
        </div>
        {allTransfers.length > 0 && (
          <span className="badge badge-accent">{completeCount} complete</span>
        )}
      </header>

      {/* Empty state */}
      {allTransfers.length === 0 && (
        <div className="empty-state py-14">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-accent-dim">
            <ArrowLeftRightIcon size={22} className="text-accent-light" />
          </div>
          <p className="text-sm font-medium text-text">No transfer history available</p>
          <p className="text-xs text-text-faint mt-1 max-w-xs mx-auto">
            Files you send to or receive from nearby devices will show up here, along with their
            progress and status.
          </p>
        </div>
      )}

      {/* Transfer list */}
      {allTransfers.length > 0 && (
        <section className="card">
          <div className="space-y-2">
            {allTransfers.map((t) => {
              const meta = STATUS_META[t.status] ?? STATUS_META.completed;
              const progress = t.sizeBytes > 0 ? (t.transferredBytes / t.sizeBytes) * 100 : 0;
              const canResume = resumeableIds.has(t.id);

              return (
                <div
                  key={t.id}
                  className="flex flex-col sm:flex-row sm:items-center gap-2 p-3 rounded-lg bg-bg-card-hover border border-border"
                >
                  <div className="flex items-center gap-3 flex-1 min-w-0">
                    <div className={`${meta.className} flex-shrink-0`}>{meta.icon}</div>

                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">{t.filename}</p>
                      <p className="text-xs text-text-muted">
                        {t.direction === "sent" ? "To" : "From"} {t.deviceName}
                      </p>
                    </div>
                  </div>

                  {t.status === "transferring" && (
                    <div className="flex items-center gap-3 w-full sm:w-auto">
                      <div className="flex-1 sm:w-32 h-1.5 rounded-full bg-bg overflow-hidden">
                        <div
                          className="h-full rounded-full bg-accent-gradient transition-all duration-300"
                          style={{ width: `${Math.min(progress, 100)}%` }}
                        />
                      </div>
                      <span className="text-xs text-text-muted font-mono whitespace-nowrap">
                        {formatBytes(t.speedMBps * 1024 * 1024)}/s
                      </span>
                    </div>
                  )}

                  <div className="flex items-center gap-3 flex-shrink-0">
                    <span className={`text-xs font-medium ${meta.className}`}>{meta.label}</span>
                    <span className="text-xs text-text-muted font-mono">
                      {formatBytes(t.sizeBytes)}
                    </span>
                    {canResume && (
                      <button
                        type="button"
                        className="btn btn-secondary text-xs px-2 py-1 inline-flex items-center gap-1"
                        onClick={() => onResume(t.id)}
                      >
                        <PlayIcon size={12} />
                        Resume
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      )}
    </div>
  );
}