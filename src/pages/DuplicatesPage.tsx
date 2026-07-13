import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";
import { CheckCircle2, ExternalLink, Files, FolderOpen, LoaderCircle, Search, ShieldCheck, StopCircle, Trash2, TriangleAlert } from "lucide-react";
import { DuplicateCleanupDialog } from "../components/duplicates/DuplicateCleanupDialog";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { commands } from "../ipc/commands";
import { formatBytes } from "../lib/utils";
import { useDuplicateStore } from "../stores/duplicateStore";
import { useSettingsStore } from "../stores/settingsStore";

export function DuplicatesPage() {
  const duplicate = useDuplicateStore();
  const { settings, load: loadSettings } = useSettingsStore();
  const [roots, setRoots] = useState<string[]>([]);
  const [minimumMiB, setMinimumMiB] = useState(10);
  const [byteVerify, setByteVerify] = useState(false);

  useEffect(() => {
    if (!settings) void loadSettings();
  }, [loadSettings, settings]);
  useEffect(() => {
    if (!settings) return;
    setMinimumMiB(Math.max(1, Math.round(settings.duplicateMinimumSizeBytes / 1_048_576)));
    setByteVerify(settings.duplicateVerificationMode === "byteForByte");
    if (roots.length === 0 && settings.projectRoots.length > 0) setRoots(settings.projectRoots.slice(0, 8));
  }, [roots.length, settings]);

  const scanning = ["starting", "running", "cancelling"].includes(duplicate.status);
  const cleaning = duplicate.status === "cleaning";
  const selectedCount = duplicate.groups.reduce((total, group) => total + group.copies.filter((copy) => duplicate.selectedCopyIds.has(copy.id)).length, 0);
  const selectedBytes = duplicate.groups.reduce((total, group) => total + group.copies.filter((copy) => duplicate.selectedCopyIds.has(copy.id)).length * group.fileSize, 0);
  const orderedGroups = useMemo(() => [...duplicate.groups].sort((left, right) => right.reclaimableBytes - left.reclaimableBytes), [duplicate.groups]);

  const chooseFolders = async () => {
    const selected = await open({ directory: true, multiple: true, title: "Choose folders for duplicate analysis" });
    if (!selected) return;
    setRoots((Array.isArray(selected) ? selected : [selected]).slice(0, 8));
  };

  return (
    <div className="mx-auto max-w-6xl px-8 py-8">
      <div className="flex items-end justify-between gap-6">
        <div><p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Content-verified analysis</p><h1 className="text-3xl font-semibold tracking-tight">Duplicates</h1><p className="mt-2 max-w-2xl text-sm leading-6 text-muted">Choose narrow folders. DiskSage filters by size, samples content, then fully hashes matching candidates locally. Symbolic links and filesystem boundaries are skipped.</p></div>
        {duplicate.summary?.phase === "completed" && <div className="text-right"><p className="text-xs text-muted">Recoverable from duplicates</p><p className="mt-1 text-2xl font-semibold">{formatBytes(duplicate.summary.reclaimableBytes)}</p></div>}
      </div>

      {duplicate.error && <Card className="mt-6 flex gap-3 border-amber-400/20 p-5" role="alert"><TriangleAlert className="shrink-0 text-amber-300" size={19} /><div><p className="font-semibold">Stopped safely</p><p className="mt-1 text-sm text-muted">{duplicate.error.message}</p>{duplicate.error.path && <p className="mt-2 truncate font-mono text-xs text-muted">{duplicate.error.path}</p>}</div></Card>}
      {duplicate.cleanupSummary && <Card className="mt-6 flex items-start gap-3 p-5"><CheckCircle2 className="shrink-0 text-sage-300" size={20} /><div><p className="font-semibold">Moved {duplicate.cleanupSummary.successCount} of {duplicate.cleanupSummary.selectedCount} duplicate copies to Trash</p><p className="mt-1 text-sm text-muted">{duplicate.cleanupSummary.skippedCount} skipped · {duplicate.cleanupSummary.failureCount} failed. Kept copies were never selected.</p></div></Card>}

      <Card className="mt-7 p-6">
        <div className="flex items-start justify-between gap-6"><div><h2 className="font-semibold">Scan folders</h2><p className="mt-1 text-sm text-muted">Folder access is explicit for this scan and is not persisted by the picker.</p></div><Button disabled={scanning || cleaning} onClick={() => void chooseFolders()} variant="secondary"><FolderOpen size={16} />Choose folders</Button></div>
        <div className="mt-4 min-h-20 rounded-xl border border-line bg-canvas/45 p-3">
          {roots.length ? roots.map((root) => <div className="flex items-center justify-between gap-3 py-1.5" key={root}><span className="truncate font-mono text-xs text-muted">{root}</span><button aria-label={`Remove ${root}`} className="text-xs text-muted hover:text-ink" disabled={scanning} onClick={() => setRoots((current) => current.filter((item) => item !== root))}>Remove</button></div>) : <div className="grid min-h-14 place-items-center text-sm text-muted">No folders selected.</div>}
        </div>
        <div className="mt-5 grid grid-cols-[220px_1fr_auto] items-end gap-5">
          <label className="grid gap-2 text-sm"><span className="font-medium">Minimum file size</span><div className="relative"><input className="control pr-12" disabled={scanning} min={1} max={1_048_576} type="number" value={minimumMiB} onChange={(event) => setMinimumMiB(Number(event.target.value))} /><span className="absolute right-3 top-3 text-xs text-muted">MiB</span></div></label>
          <label className="flex min-h-11 items-center justify-between gap-4 rounded-xl border border-line bg-white/[0.025] px-4 text-sm"><span><span className="font-medium">Byte-for-byte verification</span><span className="ml-2 text-xs text-muted">Slower, after full hash</span></span><input className="size-4 accent-[#3abb8b]" checked={byteVerify} disabled={scanning} onChange={(event) => setByteVerify(event.target.checked)} type="checkbox" /></label>
          {scanning ? <Button disabled={duplicate.status === "cancelling"} onClick={() => void duplicate.cancel()} variant="secondary">{duplicate.status === "cancelling" ? <LoaderCircle className="animate-spin" size={16} /> : <StopCircle size={16} />}{duplicate.status === "cancelling" ? "Cancelling…" : "Cancel"}</Button> : <Button disabled={!roots.length || minimumMiB < 1 || cleaning} onClick={() => void duplicate.start(roots, minimumMiB * 1_048_576, byteVerify)}><Search size={16} />Start duplicate scan</Button>}
        </div>
      </Card>

      {(duplicate.progress || scanning) && <Card className="mt-6 overflow-hidden"><div className="flex items-center justify-between border-b border-line px-6 py-4"><div><p className="text-xs uppercase tracking-[0.14em] text-muted">Duplicate scan</p><h2 className="mt-1 font-semibold capitalize">{phaseLabel(duplicate.progress?.phase ?? "discovering")}</h2></div>{scanning && <LoaderCircle className="animate-spin text-sage-300" size={20} />}</div><div className="grid grid-cols-5 gap-px bg-line"><Metric label="Files" value={(duplicate.progress?.filesScanned ?? 0).toLocaleString()} /><Metric label="Candidates" value={(duplicate.progress?.candidateFiles ?? 0).toLocaleString()} /><Metric label="Hashed" value={formatBytes(duplicate.progress?.bytesHashed ?? 0)} /><Metric label="Groups" value={(duplicate.progress?.groupsFound ?? 0).toLocaleString()} /><Metric label="Wasted" value={formatBytes(duplicate.progress?.reclaimableBytes ?? 0)} /></div><p className="truncate px-6 py-3 font-mono text-xs text-muted" aria-live="polite">{duplicate.progress?.currentPath ?? "Preparing the next stage…"}</p></Card>}

      {duplicate.cleanupProgress && cleaning && <Card className="mt-6 border-sage-400/20 p-5"><div className="flex items-center justify-between gap-5"><div><p className="font-semibold">Revalidating and moving duplicates to Trash</p><p className="mt-1 max-w-xl truncate font-mono text-xs text-muted">{duplicate.cleanupProgress.currentPath ?? "Finalizing local history…"}</p></div><p className="font-semibold tabular-nums">{duplicate.cleanupProgress.completedItems} / {duplicate.cleanupProgress.totalItems}</p></div><div className="mt-4 h-1.5 overflow-hidden rounded-full bg-white/5"><div className="h-full bg-sage-400" style={{ width: `${duplicate.cleanupProgress.totalItems ? (duplicate.cleanupProgress.completedItems / duplicate.cleanupProgress.totalItems) * 100 : 0}%` }} /></div></Card>}

      {duplicate.summary?.phase === "completed" && orderedGroups.length === 0 ? <Card className="mt-6 grid min-h-56 place-items-center text-center"><div><Files className="mx-auto text-muted" size={34} /><h2 className="mt-4 font-semibold">No content-identical groups found</h2><p className="mt-2 text-sm text-muted">Same-size files with different content were excluded.</p></div></Card> : orderedGroups.length > 0 ? <div className="mt-6 space-y-4 pb-24">{orderedGroups.map((group, index) => {
        const keepId = duplicate.keepByGroup[group.id] ?? group.recommendedKeepId;
        return <Card className="overflow-hidden shadow-none" key={group.id}><div className="flex items-center justify-between border-b border-line px-5 py-4"><div><h2 className="font-semibold">Group {index + 1} · {group.copies.length} identical copies</h2><p className="mt-1 text-xs text-muted">{formatBytes(group.fileSize)} each · {formatBytes(group.reclaimableBytes)} reclaimable · {group.byteForByteVerified ? "byte verified" : "BLAKE3 verified"}</p></div><span className="rounded-full bg-sage-400/10 px-3 py-1 text-xs text-sage-100">Keep one</span></div><div className="divide-y divide-line">{group.copies.map((copy) => {
          const keep = copy.id === keepId;
          return <div className="flex items-center gap-4 px-5 py-4" key={copy.id}><input aria-label={keep ? `Keep ${copy.displayPath}` : `Move ${copy.displayPath} to Trash`} checked={!keep && duplicate.selectedCopyIds.has(copy.id)} className="size-4 accent-emerald-400" disabled={keep || cleaning} onChange={() => duplicate.toggleTrash(copy.id)} type="checkbox" /><div className="min-w-0 flex-1"><p className="truncate font-mono text-xs text-ink">{copy.displayPath}</p><p className="mt-1 text-xs text-muted">{copy.modifiedAt ? new Date(copy.modifiedAt).toLocaleString() : "Modified time unavailable"}{copy.owner ? ` · ${copy.owner}` : ""}</p></div><label className="flex items-center gap-2 text-xs text-muted"><input checked={keep} className="accent-emerald-400" disabled={cleaning} name={`keep-${group.id}`} onChange={() => duplicate.setKeep(group.id, copy.id)} type="radio" />Keep</label><Button onClick={() => void commands.revealDuplicate(group.scanId, group.id, copy.id)} variant="ghost"><ExternalLink size={14} />Reveal</Button></div>;
        })}</div><div className="flex items-start gap-2 border-t border-line bg-white/[0.02] px-5 py-3 text-xs text-muted"><ShieldCheck className="mt-0.5 shrink-0 text-sage-300" size={14} /><span>{group.keepReason} You can choose another keep copy; the backend still enforces at least one survivor.</span></div></Card>;
      })}</div> : null}

      {selectedCount > 0 && duplicate.status !== "review" && !scanning && <div className="sticky bottom-5 z-20 mt-5 flex items-center justify-between rounded-2xl border border-sage-400/25 bg-[#0a1714]/95 p-4 shadow-2xl backdrop-blur"><div><p className="font-semibold">{selectedCount} duplicate {selectedCount === 1 ? "copy" : "copies"} selected</p><p className="mt-1 text-xs text-muted">{formatBytes(selectedBytes)} will be re-hashed and reviewed before Trash.</p></div>{cleaning ? <Button onClick={() => void duplicate.cancelCleanup()} variant="secondary"><StopCircle size={16} />Cancel remaining</Button> : <Button disabled={duplicate.status === "planning"} onClick={() => void duplicate.createPlan()}><Trash2 size={16} />{duplicate.status === "planning" ? "Creating plan…" : "Review duplicate cleanup"}</Button>}</div>}

      {duplicate.plan && (duplicate.status === "review" || cleaning) && <DuplicateCleanupDialog busy={cleaning} onCancel={duplicate.dismissPlan} onConfirm={() => void duplicate.executePlan()} plan={duplicate.plan} />}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) { return <div className="bg-panel px-5 py-4"><p className="text-xs text-muted">{label}</p><p className="mt-1 font-semibold tabular-nums">{value}</p></div>; }
function phaseLabel(phase: string) { return phase.replace(/([A-Z])/g, " $1").toLowerCase(); }
