import { useState } from "react";
import { Database, PackageX, ShieldCheck, TriangleAlert, X } from "lucide-react";
import { useModalFocus } from "../../hooks/useModalFocus";
import type { ApplicationUninstallPlan } from "../../ipc/types";
import { formatBytes } from "../../lib/utils";
import { Button } from "../ui/Button";

type Props = {
  busy: boolean;
  plan: ApplicationUninstallPlan;
  onCancel: () => void;
  onConfirm: (selectedRelatedItemIds: string[], typedConfirmation?: string) => void;
};

export function ApplicationUninstallDialog({ busy, plan, onCancel, onConfirm }: Props) {
  const [acknowledged, setAcknowledged] = useState(false);
  const [selectedRelatedItemIds, setSelectedRelatedItemIds] = useState(
    () => new Set(plan.relatedItems.filter((item) => item.defaultSelected).map((item) => item.id)),
  );
  const [typedConfirmation, setTypedConfirmation] = useState("");
  const modal = useModalFocus(onCancel);
  const bytes = plan.application.allocatedSize ?? plan.application.logicalSize;
  const complete = plan.mode === "complete";
  const deep = plan.mode === "deepCleanup";
  const hasRelatedData = complete || deep;
  const selectedItems = plan.relatedItems.filter((item) => selectedRelatedItemIds.has(item.id));
  const confirmationMatches = !deep || typedConfirmation === plan.requiredConfirmationPhrase;

  return (
    <div
      className="fixed inset-0 z-50 grid place-items-center bg-black/65 p-6"
      onMouseDown={(event) => {
        if (!busy && event.currentTarget === event.target) onCancel();
      }}
      role="presentation"
    >
      <div
        aria-labelledby="application-uninstall-title"
        aria-modal="true"
        className={`max-h-[calc(100vh-2rem)] w-full overflow-y-auto rounded-2xl border border-line bg-panel p-6 shadow-2xl ${deep ? "max-w-2xl" : "max-w-xl"}`}
        onKeyDown={modal.onKeyDown}
        ref={modal.ref as React.RefObject<HTMLDivElement>}
        role="dialog"
      >
        <div className="flex items-start justify-between gap-5">
          <div className="flex gap-3">
            <div className="grid size-11 shrink-0 place-items-center rounded-xl bg-amber-400/10 text-amber-300">
              <PackageX size={21} />
            </div>
            <div>
              <h2 className="text-lg font-semibold" id="application-uninstall-title">
                {deep ? `Deep cleanup for ${plan.application.name}?` : complete ? `App + identified data for ${plan.application.name}?` : `Move ${plan.application.name} to Trash?`}
              </h2>
              <p className="mt-1 text-sm text-muted">{hasRelatedData ? `Review the app and ${plan.relatedItems.length} related ${plan.relatedItems.length === 1 ? "item" : "items"} before continuing.` : "Review the exact application bundle before continuing."}</p>
            </div>
          </div>
          <button
            aria-label="Close uninstall review"
            className="rounded-lg p-2 text-muted hover:bg-white/5 hover:text-ink"
            disabled={busy}
            onClick={onCancel}
          >
            <X size={18} />
          </button>
        </div>

        <div className="mt-5 rounded-xl border border-line bg-canvas/55 p-4">
          <div className="flex items-center justify-between gap-4">
            <div className="min-w-0">
              <p className="font-semibold">{plan.application.name}</p>
              <p className="mt-1 truncate font-mono text-xs text-muted">{plan.application.displayPath}</p>
            </div>
            <p className="shrink-0 font-semibold tabular-nums">{formatBytes(bytes)}</p>
          </div>
          <p className="mt-3 text-xs text-muted">
            {plan.application.version ? `Version ${plan.application.version} · ` : ""}
            {plan.application.scope === "user" ? "User application" : "Shared application"}
          </p>
        </div>

        {hasRelatedData && (
          <div className="mt-4 overflow-hidden rounded-xl border border-line">
            <div className="flex items-center justify-between bg-white/[0.025] px-4 py-3">
              <p className="flex items-center gap-2 text-sm font-semibold"><Database className={deep ? "text-red-300" : "text-amber-300"} size={16} />{deep ? "Identified + expert candidates" : "Identified data"}</p>
              <p className="text-xs text-muted">{deep ? `${selectedItems.length} of ${plan.relatedItems.length} selected` : formatBytes(Math.max(0, plan.totalExpectedBytes - bytes))}</p>
            </div>
            {plan.relatedItems.length > 0 ? (
              <div className="max-h-32 divide-y divide-line overflow-y-auto">
                {plan.relatedItems.map((item) => (
                  <div className="flex items-start gap-3 px-4 py-3" key={item.path}>
                    {deep && <input aria-label={`Select ${item.displayPath}`} checked={selectedRelatedItemIds.has(item.id)} className="mt-0.5 size-4 shrink-0 accent-red-400" disabled={busy} onChange={() => setSelectedRelatedItemIds((current) => { const next = new Set(current); if (next.has(item.id)) next.delete(item.id); else next.add(item.id); return next; })} type="checkbox" />}
                    <div className="min-w-0 flex-1"><p className="flex flex-wrap items-center gap-2 text-xs font-medium">{item.category}<span className={`rounded-full px-2 py-0.5 text-[10px] ${item.confidence === "ambiguous" ? "bg-red-400/10 text-red-200" : "bg-sage-400/10 text-sage-200"}`}>{item.confidence}</span>{item.mayContainUserData && <span className="rounded-full bg-amber-400/10 px-2 py-0.5 text-[10px] text-amber-200">may contain user data</span>}</p><p className="mt-1 truncate font-mono text-[11px] text-muted">{item.displayPath}</p>{deep && <p className="mt-1 text-[11px] leading-4 text-muted">{item.reason}</p>}</div>
                    <p className="shrink-0 text-xs tabular-nums text-muted">{formatBytes(item.allocatedSize ?? item.logicalSize)}</p>
                  </div>
                ))}
              </div>
            ) : <p className="px-4 py-4 text-sm text-muted">No exact app-specific Library items were found. Only the app bundle will move.</p>}
          </div>
        )}

        <div className={`mt-4 flex gap-3 rounded-xl p-3 text-sm ${deep ? "border border-red-400/25 bg-red-400/[0.05]" : complete ? "border border-amber-400/20 bg-amber-400/[0.05]" : "border border-sage-400/20 bg-sage-400/[0.06]"}`}>
          {hasRelatedData ? <TriangleAlert className={`mt-0.5 shrink-0 ${deep ? "text-red-300" : "text-amber-300"}`} size={17} /> : <ShieldCheck className="mt-0.5 shrink-0 text-sage-300" size={17} />}
          <p className="leading-6 text-muted">
            {deep ? <>Only checked paths move. Ambiguous items are unchecked initially because they may contain personal work or data shared with other apps.</> : complete ? <>Only identified items listed above move to Trash. Ambiguous Documents, projects, and shared Group Containers remain excluded.</> : <>Only this <span className="text-ink">.app bundle</span> moves to macOS Trash. DiskSage does not remove its preferences, documents, caches, or Application Support data.</>}
          </p>
        </div>

        <label className="mt-4 flex cursor-pointer items-start gap-3 rounded-xl border border-line p-3 text-sm">
          <input
            checked={acknowledged}
            className="mt-0.5 size-4 accent-emerald-400"
            disabled={busy}
            onChange={(event) => setAcknowledged(event.target.checked)}
            type="checkbox"
          />
          <span>{hasRelatedData ? "I reviewed every selected path and understand related local data may be removed." : "I reviewed the application and understand it will stop working from this location."}</span>
        </label>

        {deep && plan.requiredConfirmationPhrase && <label className="mt-4 grid gap-2 text-sm"><span>Type <strong className="text-red-200">{plan.requiredConfirmationPhrase}</strong> to enable Expert cleanup</span><input autoComplete="off" className="control font-mono" disabled={busy} onChange={(event) => setTypedConfirmation(event.target.value)} placeholder={plan.requiredConfirmationPhrase} value={typedConfirmation} /></label>}

        <div className="sticky bottom-0 -mx-6 -mb-6 mt-6 flex justify-end gap-3 border-t border-line bg-panel/95 px-6 py-4 backdrop-blur">
          <Button disabled={busy} onClick={onCancel} variant="secondary">Cancel</Button>
          <Button disabled={!acknowledged || !confirmationMatches || busy} onClick={() => onConfirm(deep ? [...selectedRelatedItemIds] : [], deep ? typedConfirmation : undefined)} variant="destructive">
            <PackageX size={16} />{busy ? "Moving to Trash…" : deep ? `Move app + ${selectedItems.length} selected items` : complete ? `Move app + ${plan.relatedItems.length} items to Trash` : "Move application to Trash"}
          </Button>
        </div>
      </div>
    </div>
  );
}
