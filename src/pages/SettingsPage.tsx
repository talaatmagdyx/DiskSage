import { useEffect, useState } from "react";
import { AlertTriangle, Download, LoaderCircle, Palette, RotateCcw, Save, ShieldCheck } from "lucide-react";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import type { AppSettings } from "../ipc/types";
import { useSettingsStore } from "../stores/settingsStore";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import { toast } from "../stores/toastStore";

export function SettingsPage() {
  const { settings, status, error, load, save } = useSettingsStore();
  const [draft, setDraft] = useState<AppSettings | null>(null);
  const [exporting, setExporting] = useState(false);

  useEffect(() => { void load(); }, [load]);
  useEffect(() => { setDraft(settings); }, [settings]);

  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setDraft((current) => current ? { ...current, [key]: value } : current);
  };

  const submit = async () => {
    if (!draft) return;
    await save(draft);
    if (useSettingsStore.getState().status === "ready") toast({ tone: "success", title: "Settings saved", message: "Your local preferences are now active." });
  };

  const exportDiagnostics = async () => {
    setExporting(true);
    try {
      const result = await commands.exportDiagnostics();
      toast({ tone: "success", title: "Diagnostics exported", message: `A redacted report was created and revealed in Finder: ${result.path}` });
    } catch (error) {
      const normalized = normalizeCommandError(error);
      toast({ tone: "error", title: "Diagnostics export failed", message: normalized.message });
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="mx-auto max-w-5xl px-8 py-8">
      <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Preferences</p>
      <h1 className="text-3xl font-semibold tracking-tight">Settings</h1>
      <p className="mt-2 text-sm text-muted">Stored locally in the operating system application-data directory.</p>

      {status === "loading" && !draft ? (
        <Card className="mt-8 grid min-h-60 place-items-center"><LoaderCircle className="animate-spin text-muted" aria-label="Loading settings" /></Card>
      ) : error && !draft ? (
        <Card className="mt-8 border-amber-400/20 p-6" role="alert"><p>{error.message}</p><Button className="mt-4" variant="secondary" onClick={() => void load()}><RotateCcw size={16} />Try again</Button></Card>
      ) : draft ? (
        <form className="mt-8 space-y-5" onSubmit={(event) => { event.preventDefault(); void submit(); }}>
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
            <div className="mt-5 grid grid-cols-4 gap-4">
              <Field label="Large file (GiB)"><input className="control" type="number" min={0.01} step={0.25} value={draft.largeFileThresholdBytes / 1_073_741_824} onChange={(event) => update("largeFileThresholdBytes", Number(event.target.value) * 1_073_741_824)} /></Field>
              <Field label="Very large (GiB)"><input className="control" type="number" min={0.02} step={0.5} value={draft.veryLargeFileThresholdBytes / 1_073_741_824} onChange={(event) => update("veryLargeFileThresholdBytes", Number(event.target.value) * 1_073_741_824)} /></Field>
              <Field label="Huge file (GiB)"><input className="control" type="number" min={0.03} step={1} value={draft.hugeFileThresholdBytes / 1_073_741_824} onChange={(event) => update("hugeFileThresholdBytes", Number(event.target.value) * 1_073_741_824)} /></Field>
              <Field label="Old installer (days)"><input className="control" type="number" min={30} max={3650} value={draft.oldFileThresholdDays} onChange={(event) => update("oldFileThresholdDays", Number(event.target.value))} /></Field>
            </div>
            <div className="mt-5 grid grid-cols-2 gap-3">
              <Toggle checked={draft.scanHiddenFiles} onChange={(value) => update("scanHiddenFiles", value)} label="Include hidden files" />
              <Toggle checked={draft.scanExternalDrives} onChange={(value) => update("scanExternalDrives", value)} label="Include external drives" />
            </div>
            <div className="mt-5 grid grid-cols-2 gap-5">
              <Field label="Duplicate minimum size (MiB)">
                <input className="control" type="number" min={1} max={1_048_576} value={Math.max(1, Math.round(draft.duplicateMinimumSizeBytes / 1_048_576))} onChange={(event) => update("duplicateMinimumSizeBytes", Number(event.target.value) * 1_048_576)} />
              </Field>
              <Field label="Duplicate verification">
                <select className="control" value={draft.duplicateVerificationMode} onChange={(event) => update("duplicateVerificationMode", event.target.value as AppSettings["duplicateVerificationMode"])}>
                  <option value="fullHash">Full BLAKE3 hash</option>
                  <option value="byteForByte">Full hash + byte-for-byte</option>
                </select>
              </Field>
            </div>
            <div className="mt-4 flex items-center gap-2 text-xs text-muted"><ShieldCheck size={14} aria-hidden="true" /> Symlink following is locked off by the backend contract.</div>
            <Field label="Project roots for Developer Scan">
              <textarea className="control mt-4 min-h-28 font-mono text-xs" placeholder="/Users/you/Projects&#10;/Users/you/Work" value={draft.projectRoots.join("\n")} onChange={(event) => update("projectRoots", event.target.value.split("\n").map((value) => value.trim()).filter(Boolean))} />
            </Field>
            <p className="mt-2 text-xs text-muted">One absolute path per line. DiskSage detects project indicators before considering context-sensitive artifacts.</p>
          </Card>

          <Card className="p-6">
            <div className="flex items-start gap-3"><Palette className="mt-0.5 text-sage-300" size={19} aria-hidden="true" /><div><h2 className="font-semibold">Appearance and accessibility</h2><p className="mt-1 text-sm text-muted">System follows your operating-system color preference.</p></div></div>
            <div className="mt-5 grid grid-cols-2 gap-5">
              <Field label="Theme"><select className="control" value={draft.theme} onChange={(event) => update("theme", event.target.value as AppSettings["theme"])}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></Field>
              <Toggle checked={draft.diagnosticLogging} onChange={(value) => update("diagnosticLogging", value)} label="Diagnostic logging" />
            </div>
            <div className="mt-3 grid grid-cols-2 gap-3">
              <Toggle checked={draft.reducedMotion} onChange={(value) => update("reducedMotion", value)} label="Reduced motion" />
            </div>
          </Card>

          <Card className="p-6">
            <div className="flex items-start justify-between gap-6"><div><h2 className="font-semibold">Privacy and diagnostics</h2><p className="mt-1 max-w-2xl text-sm leading-6 text-muted">Export a local JSON report with app and platform details, aggregate counts, error codes, and redacted configuration. File paths, filenames, hashes, file contents, project-root values, and cleanup item details are excluded.</p></div><Button type="button" variant="secondary" disabled={exporting} onClick={() => void exportDiagnostics()}>{exporting ? <LoaderCircle className="animate-spin" size={16} /> : <Download size={16} />}{exporting ? "Exporting…" : "Export diagnostics"}</Button></div>
          </Card>

          <Card className="border-amber-400/15 p-6">
            <div className="flex gap-3">
              <AlertTriangle className="mt-0.5 text-amber-300" size={19} aria-hidden="true" />
              <div>
                <h2 className="font-semibold">Advanced cleanup</h2>
                <p className="mt-1 text-sm leading-6 text-muted">Permanent deletion bypasses Trash and cannot be undone. It still requires an immutable plan, native confirmation, and immediate backend revalidation.</p>
              </div>
            </div>
            <div className="mt-4">
              <Toggle checked={draft.permanentDeletionEnabled} onChange={(value) => update("permanentDeletionEnabled", value)} label="Enable permanent deletion" />
            </div>
            {draft.permanentDeletionEnabled && <p className="mt-3 rounded-xl border border-red-400/25 bg-red-400/10 p-3 text-xs leading-5 text-red-100" role="status">Warning: a separate permanent-delete action will appear for cleanup-authorized findings. Expert-risk plans additionally require an exact typed phrase. No item is deleted when this setting is enabled.</p>}
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
