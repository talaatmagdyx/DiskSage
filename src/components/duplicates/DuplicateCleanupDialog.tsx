import { AlertTriangle, CopyCheck, ShieldCheck, Trash2, X } from "lucide-react";
import type { DuplicateCleanupPlan } from "../../ipc/types";
import { formatBytes } from "../../lib/utils";
import { Button } from "../ui/Button";
import { useModalFocus } from "../../hooks/useModalFocus";

type Props = {
  plan: DuplicateCleanupPlan;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
};

export function DuplicateCleanupDialog({ plan, busy, onCancel, onConfirm }: Props) {
  const modal = useModalFocus(onCancel, !busy);
  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/70 p-6" role="presentation">
      <section ref={modal.ref} onKeyDown={modal.onKeyDown} aria-labelledby="duplicate-review-title" aria-modal="true" className="w-full max-w-2xl rounded-2xl border border-line bg-panel p-6 shadow-2xl" role="dialog">
        <div className="flex items-start justify-between gap-4">
          <div className="flex gap-3">
            <div className="grid size-11 shrink-0 place-items-center rounded-xl bg-sage-400/10 text-sage-300"><CopyCheck size={21} /></div>
            <div><p className="text-xs font-semibold uppercase tracking-[0.16em] text-sage-300">Immutable duplicate plan</p><h2 className="mt-1 text-xl font-semibold" id="duplicate-review-title">Keep one, move reviewed copies to Trash</h2></div>
          </div>
          <Button aria-label="Close duplicate cleanup review" disabled={busy} onClick={onCancel} variant="ghost"><X size={18} /></Button>
        </div>

        <div className="mt-6 grid grid-cols-3 gap-3">
          <Metric label="Copies moving" value={plan.items.length.toLocaleString()} />
          <Metric label="Copies kept" value={plan.keptCopyCount.toLocaleString()} />
          <Metric label="Selected size" value={formatBytes(plan.expectedReclaimableBytes)} />
        </div>

        <div className="mt-4 max-h-52 space-y-2 overflow-y-auto rounded-xl border border-line bg-canvas/45 p-3">
          {plan.items.map((item) => <div className="rounded-lg px-2 py-2 text-xs" key={item.copyId}><p className="truncate font-mono text-muted">Trash: {item.path}</p><p className="mt-1 truncate font-mono text-sage-200">Keep: {item.keepPath}</p></div>)}
        </div>

        <div className="mt-4 flex gap-3 rounded-xl border border-amber-400/20 bg-amber-400/[0.06] p-4 text-sm text-muted">
          <AlertTriangle className="mt-0.5 shrink-0 text-amber-300" size={18} />
          <p>The backend re-hashes the keep copy and every selected copy immediately before moving it. If content, metadata, path resolution, or protection changes, that file is skipped. At least one copy remains in every planned group.</p>
        </div>
        <p className="mt-3 flex items-center gap-2 text-xs text-muted"><ShieldCheck size={14} />Trash can be restored; space may not increase until Trash is emptied.</p>
        <div className="mt-6 flex justify-end gap-3"><Button disabled={busy} onClick={onCancel} variant="secondary">Keep all copies</Button><Button disabled={busy} onClick={onConfirm}><Trash2 size={16} />{busy ? "Starting…" : "Move selected to Trash"}</Button></div>
      </section>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return <div className="rounded-xl border border-line bg-canvas/45 p-4"><p className="text-xs text-muted">{label}</p><p className="mt-1 text-xl font-semibold">{value}</p></div>;
}
