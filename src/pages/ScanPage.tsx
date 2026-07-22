import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle2, Clock3, Code2, FolderOpen, Gauge, LoaderCircle, Search, ShieldCheck, StopCircle } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { formatBytes } from "../lib/utils";
import { useScanStore } from "../stores/scanStore";

const profileIcons = { quick: Gauge, developer: Code2, fullAnalysis: Search, custom: Search } as const;

export function ScanPage() {
  const { profiles, status, progress, summary, error, loadProfiles, start, cancel } = useScanStore();
  const active = ["starting", "running", "cancelling"].includes(status);
  const [customOpen, setCustomOpen] = useState(false);
  const [customRoots, setCustomRoots] = useState<string[]>([]);
  const [customExclusions, setCustomExclusions] = useState("");
  const [minimumMiB, setMinimumMiB] = useState(10);
  const [maximumDepth, setMaximumDepth] = useState(12);
  const [largeFiles, setLargeFiles] = useState(true);
  const [oldFiles, setOldFiles] = useState(true);

  useEffect(() => {
    if (profiles.length === 0) void loadProfiles();
  }, [loadProfiles, profiles.length]);

  const chooseCustomRoots = async () => {
    const selected = await open({ directory: true, multiple: true, title: "Choose folders for Custom Scan" });
    if (selected) setCustomRoots((Array.isArray(selected) ? selected : [selected]).slice(0, 8));
  };

  const startCustom = () => {
    const enabledCategories = [largeFiles ? "largeFile" : null, oldFiles ? "oldFile" : null].filter(Boolean) as ("largeFile" | "oldFile")[];
    void start("custom", customExclusions.split("\n").map((value) => value.trim()).filter(Boolean), {
      roots: customRoots,
      enabledCategories,
      minimumFileSizeBytes: minimumMiB * 1_048_576,
      maximumDepth,
      includeHiddenFiles: false,
      includeExternalDrives: false,
    });
  };

  return (
    <div className="mx-auto max-w-6xl px-8 py-8">
      <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Targeted analysis</p>
      <h1 className="text-3xl font-semibold tracking-tight">Choose a scan</h1>
      <p className="mt-2 max-w-2xl text-sm leading-6 text-muted">
        DiskSage visits known developer locations plus project roots you explicitly configure. Symbolic links and other filesystems are skipped.
      </p>

      {error && <Card className="mt-6 border-amber-400/20 p-4 text-sm text-amber-100" role="alert">{error.message}</Card>}

      <div className="mt-7 grid grid-cols-2 gap-4">
        {profiles.map((profile) => {
          const Icon = profileIcons[profile.id];
          return (
            <Card key={profile.id} className={profile.available ? "p-5" : "p-5 opacity-60"}>
              <div className="flex items-start gap-4">
                <div className="grid size-11 shrink-0 place-items-center rounded-xl bg-sage-400/10 text-sage-300"><Icon size={20} /></div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center justify-between gap-4">
                    <h2 className="font-semibold">{profile.displayName}</h2>
                    {!profile.available && <span className="rounded-full border border-line px-2 py-1 text-[10px] uppercase tracking-wide text-muted">Later phase</span>}
                  </div>
                  <p className="mt-2 text-sm leading-5 text-muted">{profile.description}</p>
                  <p className="mt-3 flex items-center gap-2 text-xs text-muted"><Clock3 size={13} />{profile.expectedDuration}</p>
                  {profile.warning && <p className="mt-3 text-xs leading-5 text-amber-100"><AlertTriangle className="mr-1 inline" size={13} />{profile.warning}</p>}
                  <Button className="mt-5" variant={profile.id === "quick" ? "primary" : "secondary"} disabled={!profile.available || active} onClick={() => profile.id === "custom" ? setCustomOpen((value) => !value) : void start(profile.id)}>
                    {profile.id === "custom" ? "Configure Custom Scan" : `Start ${profile.displayName}`}
                  </Button>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      {customOpen && <Card className="mt-6 p-6"><div className="flex items-start justify-between gap-5"><div><h2 className="font-semibold">Custom Scan scope</h2><p className="mt-1 text-sm text-muted">Large files and old installers remain review-only. Selected folders authorize analysis, never cleanup.</p></div><Button disabled={active} onClick={() => void chooseCustomRoots()} variant="secondary"><FolderOpen size={16} />Choose folders</Button></div><div className="mt-4 rounded-xl border border-line bg-canvas/45 p-3">{customRoots.length ? customRoots.map((root) => <div className="flex items-center justify-between gap-3 py-1.5" key={root}><span className="truncate font-mono text-xs text-muted">{root}</span><button className="text-xs text-muted hover:text-ink" onClick={() => setCustomRoots((current) => current.filter((item) => item !== root))}>Remove</button></div>) : <p className="py-4 text-center text-sm text-muted">No folders selected.</p>}</div><div className="mt-5 grid grid-cols-3 gap-4"><label className="grid gap-2 text-sm"><span>Minimum file size (MiB)</span><input className="control" min={0} type="number" value={minimumMiB} onChange={(event) => setMinimumMiB(Number(event.target.value))} /></label><label className="grid gap-2 text-sm"><span>Maximum depth</span><input className="control" min={1} max={64} type="number" value={maximumDepth} onChange={(event) => setMaximumDepth(Number(event.target.value))} /></label><label className="grid gap-2 text-sm"><span>Excluded absolute paths</span><textarea className="control min-h-11 font-mono text-xs" placeholder="One per line" value={customExclusions} onChange={(event) => setCustomExclusions(event.target.value)} /></label></div><div className="mt-4 grid grid-cols-2 gap-3"><label className="flex items-center justify-between rounded-xl border border-line px-4 py-3 text-sm"><span>Large-file analysis</span><input checked={largeFiles} className="accent-emerald-400" onChange={(event) => setLargeFiles(event.target.checked)} type="checkbox" /></label><label className="flex items-center justify-between rounded-xl border border-line px-4 py-3 text-sm"><span>Old installer analysis</span><input checked={oldFiles} className="accent-emerald-400" onChange={(event) => setOldFiles(event.target.checked)} type="checkbox" /></label></div><div className="mt-5 flex justify-end"><Button disabled={active || customRoots.length === 0 || (!largeFiles && !oldFiles) || maximumDepth < 1 || maximumDepth > 64} onClick={startCustom}><Search size={16} />Start Custom Scan</Button></div></Card>}

      {(active || progress || summary) && (
        <Card className="mt-6 overflow-hidden">
          <div className="flex items-center justify-between border-b border-line px-6 py-4">
            <div>
              <p className="text-xs uppercase tracking-[0.14em] text-muted">Current scan</p>
              <h2 className="mt-1 font-semibold capitalize">{summary?.phase ?? progress?.phase ?? status}</h2>
            </div>
            {active && (
              <Button variant="secondary" disabled={status === "cancelling"} onClick={() => void cancel()}>
                {status === "cancelling" ? <LoaderCircle className="animate-spin" size={16} /> : <StopCircle size={16} />}
                {status === "cancelling" ? "Cancelling…" : "Cancel scan"}
              </Button>
            )}
            {summary?.phase === "completed" && <Link className="rounded-xl bg-sage-400 px-4 py-2.5 text-sm font-semibold text-sage-900" to="/cleanup">Review findings</Link>}
            {summary?.phase === "cancelled" && summary.profile !== "custom" && <Button onClick={() => void start(summary.profile)}><Search size={16} />Run again</Button>}
          </div>
          <div className="grid grid-cols-5 gap-px bg-line">
            <ProgressMetric label="Files" value={(summary?.filesScanned ?? progress?.filesScanned ?? 0).toLocaleString()} />
            <ProgressMetric label="Directories" value={(summary?.directoriesScanned ?? progress?.directoriesScanned ?? 0).toLocaleString()} />
            <ProgressMetric label="Examined" value={formatBytes(summary?.bytesExamined ?? progress?.bytesExamined ?? 0)} />
            <ProgressMetric label="Findings" value={(summary?.findingsCount ?? progress?.findingsCount ?? 0).toLocaleString()} />
            <ProgressMetric label="Reclaimable" value={formatBytes(summary?.reclaimableBytes ?? progress?.reclaimableBytes ?? 0)} />
          </div>
          <div className="flex min-h-12 items-center gap-3 px-6 py-3 text-xs text-muted" aria-live="polite">
            {active ? <LoaderCircle className="animate-spin text-sage-300" size={15} /> : summary?.phase === "cancelled" ? <StopCircle className="text-amber-300" size={15} /> : <CheckCircle2 className="text-sage-300" size={15} />}
            <span className="truncate font-mono">{summary?.phase === "cancelled" ? "Scan cancelled; partial findings preserved." : summary ? "Scan state saved locally." : progress?.currentPath ?? "Preparing scan…"}</span>
            {(summary?.permissionDeniedCount ?? progress?.permissionDeniedCount ?? 0) > 0 && <span className="ml-auto shrink-0 text-amber-100">{summary?.permissionDeniedCount ?? progress?.permissionDeniedCount} permission-limited</span>}
          </div>
        </Card>
      )}

      <p className="mt-5 flex items-center gap-2 text-xs text-muted"><ShieldCheck size={14} /> Scanning never triggers cleanup. Every finding requires a later review and immutable cleanup plan.</p>
    </div>
  );
}

function ProgressMetric({ label, value }: { label: string; value: string }) {
  return <div className="bg-panel px-5 py-4"><p className="text-xs text-muted">{label}</p><p className="mt-1 font-semibold tabular-nums">{value}</p></div>;
}
