import { AlertTriangle, ShieldCheck, Trash2, X } from "lucide-react";
import { Button } from "../ui/Button";
import type { CleanupPlan } from "../../ipc/types";
import { formatBytes } from "../../lib/utils";

type CleanupReviewDialogProps = {
  plan: CleanupPlan;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
};

export function CleanupReviewDialog({
  plan,
  busy,
  onCancel,
  onConfirm,
}: CleanupReviewDialogProps) {
  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/70 p-6" role="presentation">
      <section
        aria-labelledby="cleanup-review-title"
        aria-modal="true"
        className="w-full max-w-2xl rounded-2xl border border-line bg-panel p-6 shadow-2xl"
        role="dialog"
      >
        <div className="flex items-start justify-between gap-4">
          <div className="flex gap-3">
            <div className="grid size-11 shrink-0 place-items-center rounded-xl bg-sage-400/10 text-sage-300">
              <ShieldCheck size={21} />
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-[0.16em] text-sage-300">Immutable cleanup plan</p>
              <h2 className="mt-1 text-xl font-semibold" id="cleanup-review-title">Review before moving to Trash</h2>
            </div>
          </div>
          <Button aria-label="Close cleanup review" disabled={busy} onClick={onCancel} variant="ghost"><X size={18} /></Button>
        </div>

        <div className="mt-6 grid grid-cols-3 gap-3">
          <div className="rounded-xl border border-line bg-canvas/45 p-4"><p className="text-xs text-muted">Items</p><p className="mt-1 text-xl font-semibold">{plan.items.length}</p></div>
          <div className="rounded-xl border border-line bg-canvas/45 p-4"><p className="text-xs text-muted">Selected size</p><p className="mt-1 text-xl font-semibold">{formatBytes(plan.expectedReclaimableBytes)}</p></div>
          <div className="rounded-xl border border-line bg-canvas/45 p-4"><p className="text-xs text-muted">Risk</p><p className="mt-1 text-xl font-semibold text-sage-200">Safe only</p></div>
        </div>

        <div className="mt-4 max-h-52 space-y-2 overflow-y-auto rounded-xl border border-line bg-canvas/45 p-3">
          {plan.items.map((item) => (
            <div className="flex items-center justify-between gap-4 rounded-lg px-2 py-2 text-sm" key={item.findingId}>
              <span className="min-w-0 truncate font-mono text-xs text-muted">{item.path}</span>
              <span className="shrink-0 tabular-nums">{formatBytes(item.expectedSize)}</span>
            </div>
          ))}
        </div>

        <div className="mt-4 flex gap-3 rounded-xl border border-amber-400/20 bg-amber-400/[0.06] p-4 text-sm text-muted">
          <AlertTriangle className="mt-0.5 shrink-0 text-amber-300" size={18} />
          <p>Each item is revalidated immediately before it moves. Changed, missing, linked, or protected items are skipped. Trash can be restored; disk space may not increase until Trash is emptied.</p>
        </div>

        <div className="mt-6 flex justify-end gap-3">
          <Button disabled={busy} onClick={onCancel} variant="secondary">Keep everything</Button>
          <Button disabled={busy} onClick={onConfirm}><Trash2 size={16} />{busy ? "Starting…" : "Move to Trash"}</Button>
        </div>
      </section>
    </div>
  );
}
