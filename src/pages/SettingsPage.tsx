import { useEffect, useState } from "react";
import { AlertTriangle, LoaderCircle, Save, ShieldCheck } from "lucide-react";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import type { AppSettings } from "../ipc/types";
import { useSettingsStore } from "../stores/settingsStore";

export function SettingsPage() {
  const { settings, status, error, load, save } = useSettingsStore();
  const [draft, setDraft] = useState<AppSettings | null>(null);

  useEffect(() => { void load(); }, [load]);
  useEffect(() => { setDraft(settings); }, [settings]);

  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setDraft((current) => current ? { ...current, [key]: value } : current);
  };

  return (
    <div className="mx-auto max-w-5xl px-8 py-8">
      <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Preferences</p>
      <h1 className="text-3xl font-semibold tracking-tight">Settings</h1>
      <p className="mt-2 text-sm text-muted">Stored locally in the operating system application-data directory.</p>

      {status === "loading" && !draft ? (
        <Card className="mt-8 grid min-h-60 place-items-center"><LoaderCircle className="animate-spin text-muted" aria-label="Loading settings" /></Card>
      ) : error && !draft ? (
        <Card className="mt-8 border-amber-400/20 p-6" role="alert"><p>{error.message}</p></Card>
      ) : draft ? (
        <form className="mt-8 space-y-5" onSubmit={(event) => { event.preventDefault(); void save(draft); }}>
          <Card className="p-6">
            <h2 className="font-semibold">Scan defaults</h2>
            <p className="mt-1 text-sm text-muted">Conservative settings used by future scan phases.</p>
            <div className="mt-5 grid grid-cols-2 gap-5">
              <Field label="Default scan mode">
                <select value={draft.defaultScanMode} onChange={(e) => update("defaultScanMode", e.target.value as AppSettings["defaultScanMode"])} className="control">
                  <option value="quick">Quick scan</option><option value="developer">Developer scan</option><option value="fullAnalysis">Full analysis</option><option value="custom">Custom scan</option>
                </select>
              </Field>
              <Field label="Maximum filesystem workers">
                <input className="control" type="number" min={1} max={8} value={draft.maximumConcurrency} onChange={(e) => update("maximumConcurrency", Number(e.target.value))} />
              </Field>
            </div>
            <div className="mt-5 grid grid-cols-2 gap-3">
              <Toggle checked={draft.scanHiddenFiles} onChange={(value) => update("scanHiddenFiles", value)} label="Include hidden files" />
              <Toggle checked={draft.scanExternalDrives} onChange={(value) => update("scanExternalDrives", value)} label="Include external drives" />
            </div>
            <div className="mt-4 flex items-center gap-2 text-xs text-muted"><ShieldCheck size={14} aria-hidden="true" /> Symlink following is locked off by the backend contract.</div>
          </Card>

          <Card className="p-6">
            <h2 className="font-semibold">Privacy and appearance</h2>
            <div className="mt-5 grid grid-cols-2 gap-3">
              <Toggle checked={draft.diagnosticLogging} onChange={(value) => update("diagnosticLogging", value)} label="Diagnostic logging" />
              <Toggle checked={draft.reducedMotion} onChange={(value) => update("reducedMotion", value)} label="Reduced motion" />
            </div>
          </Card>

          <Card className="border-amber-400/15 p-6">
            <div className="flex gap-3">
              <AlertTriangle className="mt-0.5 text-amber-300" size={19} aria-hidden="true" />
              <div>
                <h2 className="font-semibold">Advanced cleanup</h2>
                <p className="mt-1 text-sm leading-6 text-muted">Permanent deletion is disabled and no deletion command exists in this foundation build.</p>
              </div>
            </div>
            <div className="mt-4">
              <Toggle checked={draft.permanentDeletionEnabled} onChange={(value) => update("permanentDeletionEnabled", value)} label="Record permanent-deletion preference for a future release" />
            </div>
            {draft.permanentDeletionEnabled && <p className="mt-3 rounded-xl bg-amber-300/10 p-3 text-xs leading-5 text-amber-100" role="status">Warning: permanent deletion cannot be undone. Enabling this preference does not add or execute destructive functionality.</p>}
          </Card>

          {error && <p className="text-sm text-amber-100" role="alert">{error.message}</p>}
          <div className="flex justify-end">
            <Button type="submit" disabled={status === "saving"}>
              {status === "saving" ? <LoaderCircle className="animate-spin" size={16} /> : <Save size={16} />} Save settings
            </Button>
          </div>
        </form>
      ) : null}
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return <label className="grid gap-2 text-sm"><span className="font-medium">{label}</span>{children}</label>;
}

function Toggle({ checked, onChange, label }: { checked: boolean; onChange: (value: boolean) => void; label: string }) {
  return (
    <label className="flex cursor-pointer items-center justify-between gap-4 rounded-xl border border-line bg-white/[0.025] px-4 py-3 text-sm">
      <span>{label}</span>
      <input className="size-4 accent-[#3abb8b]" type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
    </label>
  );
}

