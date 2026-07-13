import { AlertTriangle, ShieldCheck, Trash2, X } from "lucide-react";
import { Button } from "../ui/Button";
import type { CleanupPlan } from "../../ipc/types";
import { formatBytes } from "../../lib/utils";
import { useState } from "react";

type CleanupReviewDialogProps = {
  plan: CleanupPlan;
  busy: boolean;
  onCancel: () => void;
  onConfirm: (typedConfirmation?: string) => void;
};

export function CleanupReviewDialog({
  plan,
  busy,
  onCancel,
  onConfirm,
}: CleanupReviewDialogProps) {
  const [typedConfirmation, setTypedConfirmation] = useState("");
  const permanent = plan.action === "permanentDelete";
  const phraseMatches = !plan.requiredConfirmationPhrase || typedConfirmation === plan.requiredConfirmationPhrase;
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
              <p className={`text-xs font-semibold uppercase tracking-[0.16em] ${permanent ? "text-red-300" : "text-sage-300"}`}>Immutable cleanup plan</p>
              <h2 className="mt-1 text-xl font-semibold" id="cleanup-review-title">{permanent ? "Review permanent deletion" : "Review before moving to Trash"}</h2>
            </div>
          </div>
          <Button aria-label="Close cleanup review" disabled={busy} onClick={onCancel} variant="ghost"><X size={18} /></Button>
        </div>

        <div className="mt-6 grid grid-cols-3 gap-3">
          <div className="rounded-xl border border-line bg-canvas/45 p-4"><p className="text-xs text-muted">Items</p><p className="mt-1 text-xl font-semibold">{plan.items.length}</p></div>
          <div className="rounded-xl border border-line bg-canvas/45 p-4"><p className="text-xs text-muted">Selected size</p><p className="mt-1 text-xl font-semibold">{formatBytes(plan.expectedReclaimableBytes)}</p></div>
          <div className="rounded-xl border border-line bg-canvas/45 p-4"><p className="text-xs text-muted">Risk</p><p className={`mt-1 text-xl font-semibold ${permanent ? "text-red-200" : "text-sage-200"}`}>{plan.riskSummary.safe} safe · {plan.riskSummary.careful} careful · {plan.riskSummary.expert} expert</p></div>
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
          <p>{permanent ? "This action cannot be undone. Each item is revalidated immediately before deletion, and no destructive retry is attempted. Changed, missing, linked, or protected items are skipped." : "Each item is revalidated immediately before it moves. Changed, missing, linked, or protected items are skipped. Trash can be restored; disk space may not increase until Trash is emptied."}</p>
        </div>

        {plan.requiredConfirmationPhrase && <label className="mt-4 grid gap-2 text-sm"><span>Type <strong className="font-mono text-red-200">{plan.requiredConfirmationPhrase}</strong> to confirm expert-risk deletion.</span><input autoComplete="off" className="control border-red-400/30 font-mono" value={typedConfirmation} onChange={(event) => setTypedConfirmation(event.target.value)} /></label>}

        <div className="mt-6 flex justify-end gap-3">
          <Button disabled={busy} onClick={onCancel} variant="secondary">Keep everything</Button>
          <Button disabled={busy || !phraseMatches} onClick={() => onConfirm(typedConfirmation || undefined)} variant={permanent ? "destructive" : "primary"}><Trash2 size={16} />{busy ? "Starting…" : permanent ? "Permanently delete" : "Move to Trash"}</Button>
        </div>
      </section>
    </div>
  );
}
