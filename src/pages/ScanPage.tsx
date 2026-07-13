import { useEffect } from "react";
import { AlertTriangle, CheckCircle2, Clock3, Code2, Gauge, LoaderCircle, Search, ShieldCheck, StopCircle } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { formatBytes } from "../lib/utils";
import { useScanStore } from "../stores/scanStore";

const profileIcons = { quick: Gauge, developer: Code2, fullAnalysis: Search, custom: Search } as const;

export function ScanPage() {
  const { profiles, status, progress, summary, error, loadProfiles, start, cancel } = useScanStore();
  const active = ["starting", "running", "cancelling"].includes(status);

  useEffect(() => {
    if (profiles.length === 0) void loadProfiles();
  }, [loadProfiles, profiles.length]);

  return (
    <div className="mx-auto max-w-6xl px-8 py-8">
      <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Targeted analysis</p>
      <h1 className="text-3xl font-semibold tracking-tight">Choose a scan</h1>
      <p className="mt-2 max-w-2xl text-sm leading-6 text-muted">
        DiskSage only visits backend-owned cache locations for these profiles. Symbolic links and other filesystems are skipped.
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
                  <Button className="mt-5" variant={profile.id === "quick" ? "primary" : "secondary"} disabled={!profile.available || active} onClick={() => void start(profile.id)}>
                    Start {profile.displayName}
                  </Button>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

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
          </div>
          <div className="grid grid-cols-5 gap-px bg-line">
            <ProgressMetric label="Files" value={(progress?.filesScanned ?? summary?.filesScanned ?? 0).toLocaleString()} />
            <ProgressMetric label="Directories" value={(progress?.directoriesScanned ?? summary?.directoriesScanned ?? 0).toLocaleString()} />
            <ProgressMetric label="Examined" value={formatBytes(progress?.bytesExamined ?? summary?.bytesExamined ?? 0)} />
            <ProgressMetric label="Findings" value={(progress?.findingsCount ?? summary?.findingsCount ?? 0).toLocaleString()} />
            <ProgressMetric label="Reclaimable" value={formatBytes(progress?.reclaimableBytes ?? summary?.reclaimableBytes ?? 0)} />
          </div>
          <div className="flex min-h-12 items-center gap-3 px-6 py-3 text-xs text-muted" aria-live="polite">
            {active ? <LoaderCircle className="animate-spin text-sage-300" size={15} /> : <CheckCircle2 className="text-sage-300" size={15} />}
            <span className="truncate font-mono">{progress?.currentPath ?? (summary?.phase === "cancelled" ? "Scan cancelled; partial findings preserved." : "Scan state saved locally.")}</span>
            {(progress?.permissionDeniedCount ?? summary?.permissionDeniedCount ?? 0) > 0 && <span className="ml-auto shrink-0 text-amber-100">{progress?.permissionDeniedCount ?? summary?.permissionDeniedCount} permission-limited</span>}
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

