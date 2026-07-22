import { AppWindow, Database, ShieldAlert, ShieldCheck, X } from "lucide-react";
import { useModalFocus } from "../../hooks/useModalFocus";
import type { ApplicationUninstallMode, InstalledApplication } from "../../ipc/types";
import { Button } from "../ui/Button";

type Props = {
  application: InstalledApplication;
  onCancel: () => void;
  onChoose: (mode: ApplicationUninstallMode) => void;
};

export function UninstallModeDialog({ application, onCancel, onChoose }: Props) {
  const modal = useModalFocus(onCancel);
  return (
    <div
      className="fixed inset-0 z-50 grid place-items-center bg-black/65 p-6"
      onMouseDown={(event) => {
        if (event.currentTarget === event.target) onCancel();
      }}
      role="presentation"
    >
      <div
        aria-labelledby="uninstall-mode-title"
        aria-modal="true"
        className="w-full max-w-4xl rounded-2xl border border-line bg-panel p-6 shadow-2xl"
        onKeyDown={modal.onKeyDown}
        ref={modal.ref as React.RefObject<HTMLDivElement>}
        role="dialog"
      >
        <div className="flex items-start justify-between gap-5">
          <div>
            <h2 className="text-lg font-semibold" id="uninstall-mode-title">Uninstall {application.name}</h2>
            <p className="mt-1 text-sm text-muted">Choose what DiskSage should scan and prepare for Trash.</p>
          </div>
          <button aria-label="Close uninstall options" className="rounded-lg p-2 text-muted hover:bg-white/5 hover:text-ink" onClick={onCancel}><X size={18} /></button>
        </div>

        <div className="mt-6 grid grid-cols-3 gap-4">
          <button className="rounded-2xl border border-line bg-canvas/45 p-5 text-left transition-colors hover:border-sage-400/40 hover:bg-sage-400/[0.06]" onClick={() => onChoose("appOnly")}>
            <AppWindow className="text-sage-300" size={24} />
            <h3 className="mt-4 font-semibold">App only</h3>
            <p className="mt-2 text-sm leading-6 text-muted">Move only the reviewed <span className="text-ink">.app bundle</span> to Trash. Preferences, caches, and local data remain.</p>
          </button>
          <button className="rounded-2xl border border-amber-400/25 bg-amber-400/[0.04] p-5 text-left transition-colors hover:border-amber-300/50 hover:bg-amber-400/[0.08]" onClick={() => onChoose("complete")}>
            <Database className="text-amber-300" size={24} />
            <h3 className="mt-4 font-semibold">App + related data</h3>
            <p className="mt-2 text-sm leading-6 text-muted">Find exact app-specific items in your Library, then show every match for review before Trash.</p>
          </button>
          <button className="rounded-2xl border border-red-400/25 bg-red-400/[0.04] p-5 text-left transition-colors hover:border-red-300/50 hover:bg-red-400/[0.08]" onClick={() => onChoose("deepCleanup")}>
            <ShieldAlert className="text-red-300" size={24} />
            <h3 className="mt-4 font-semibold">Deep cleanup <span className="ml-1 rounded-full bg-red-400/10 px-2 py-0.5 text-[10px] uppercase text-red-200">Expert</span></h3>
            <p className="mt-2 text-sm leading-6 text-muted">Also list ambiguous Documents and shared containers, unchecked, with per-path selection and typed confirmation.</p>
          </button>
        </div>

        <div className="mt-5 flex gap-3 rounded-xl border border-sage-400/20 bg-sage-400/[0.05] p-4 text-sm text-muted">
          <ShieldCheck className="mt-0.5 shrink-0 text-sage-300" size={17} />
          <p className="leading-6">Nothing moves until the next confirmation screen. App-only and identified-data modes exclude ambiguous files; Expert mode lists narrow candidates but never selects them automatically.</p>
        </div>

        <div className="mt-6 flex justify-end"><Button onClick={onCancel} variant="secondary">Cancel</Button></div>
      </div>
    </div>
  );
}
