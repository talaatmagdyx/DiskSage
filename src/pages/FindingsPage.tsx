import { useEffect, useMemo, useState } from "react";
import {
  CheckCircle2,
  ExternalLink,
  Filter,
  FolderSearch,
  HardDriveDownload,
  Search,
  ShieldCheck,
  SquareTerminal,
  Trash2,
  TriangleAlert,
  XCircle,
} from "lucide-react";
import { CleanupReviewDialog } from "../components/cleanup/CleanupReviewDialog";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { VirtualizedList } from "../components/ui/VirtualizedList";
import { commands } from "../ipc/commands";
import type { RuleCategory } from "../ipc/types";
import { formatBytes, presentStorageSize } from "../lib/utils";
import { useCleanupStore } from "../stores/cleanupStore";
import { useFindingsStore } from "../stores/findingsStore";
import { useScanStore } from "../stores/scanStore";
import { useSettingsStore } from "../stores/settingsStore";

const categoryLabels: Record<RuleCategory, string> = {
  applicationCache: "Application cache",
  browserCache: "Browser cache",
  packageManagerCache: "Package manager",
  buildArtifact: "Build artifact",
  log: "Logs",
  installer: "Installers",
  duplicate: "Duplicates",
  largeFile: "Large files",
  oldFile: "Old files",
  container: "Containers",
  emulator: "Emulators",
};

export function FindingsPage() {
  const { findings, status, error, load } = useFindingsStore();
  const summary = useScanStore((state) => state.summary);
  const cleanup = useCleanupStore();
  const { settings, load: loadSettings } = useSettingsStore();
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<RuleCategory | "all">("all");
  const [selected, setSelected] = useState<Set<string>>(() => new Set());
  const [expandedGuides, setExpandedGuides] = useState<Set<string>>(() => new Set());

  useEffect(() => {
    if (summary?.scanId && findings.length === 0 && status === "idle") void load(summary.scanId);
  }, [findings.length, load, status, summary?.scanId]);
  useEffect(() => { if (!settings) void loadSettings(); }, [loadSettings, settings]);

  useEffect(() => {
    const existing = new Set(findings.map((finding) => finding.id));
    setSelected((current) => new Set([...current].filter((id) => existing.has(id))));
  }, [findings]);

  const categories = useMemo(() => Array.from(new Set(findings.map((finding) => finding.category))), [findings]);
  const visible = useMemo(() => findings
    .filter((finding) => category === "all" || finding.category === category)
    .filter((finding) => `${finding.displayName} ${finding.displayPath}`.toLowerCase().includes(query.toLowerCase()))
    .sort((left, right) => presentStorageSize(right.logicalSize, right.allocatedSize).displayedBytes - presentStorageSize(left.logicalSize, left.allocatedSize).displayedBytes), [category, findings, query]);
  const total = visible.reduce((sum, finding) => sum + presentStorageSize(finding.logicalSize, finding.allocatedSize).displayedBytes, 0);
  const selectedFindings = findings.filter((finding) => selected.has(finding.id));
  const selectedBytes = selectedFindings.reduce((sum, finding) => sum + presentStorageSize(finding.logicalSize, finding.allocatedSize).displayedBytes, 0);
  const selectedRisk = selectedFindings.reduce((counts, finding) => {
    counts[finding.risk] += 1;
    return counts;
  }, { safe: 0, careful: 0, expert: 0 });
  const cleanupBusy = cleanup.status === "planning" || cleanup.status === "starting" || cleanup.status === "running";

  const toggle = (findingId: string) => {
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(findingId)) next.delete(findingId); else next.add(findingId);
      return next;
    });
  };

  const createPlan = (action: "moveToTrash" | "permanentDelete") => {
    if (summary?.scanId && selected.size > 0) void cleanup.createPlan(summary.scanId, [...selected], action);
  };

  const toggleGuide = (findingId: string) => {
    setExpandedGuides((current) => {
      const next = new Set(current);
      if (next.has(findingId)) next.delete(findingId); else next.add(findingId);
      return next;
    });
  };

  return (
    <div className="mx-auto max-w-6xl px-8 py-8">
      <div className="flex items-end justify-between gap-6">
        <div>
          <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Latest scan</p>
          <h1 className="text-3xl font-semibold tracking-tight">Findings</h1>
          <p className="mt-2 text-sm text-muted">Known, regenerable cache locations with backend evidence and exact cleanup boundaries.</p>
        </div>
        <div className="text-right"><p className="text-xs text-muted">Visible allocated size</p><p className="mt-1 text-2xl font-semibold">{formatBytes(total)}</p></div>
      </div>

      {cleanup.error && <Card className="mt-6 flex items-start gap-3 border-amber-400/20 p-5" role="alert"><TriangleAlert className="shrink-0 text-amber-300" size={19} /><div><p className="font-semibold">Cleanup stopped safely</p><p className="mt-1 text-sm text-muted">{cleanup.error.message}</p></div></Card>}
      {cleanup.summary && (
        <Card className="mt-6 p-5">
          <div className="flex items-start justify-between gap-5">
              <div className="flex gap-3">{cleanup.summary.failureCount > 0 || cleanup.summary.skippedCount > 0 ? <TriangleAlert className="text-amber-300" size={21} /> : <CheckCircle2 className="text-sage-300" size={21} />}<div><h2 className="font-semibold">{cleanup.summary.action === "permanentDelete" ? "Permanently deleted" : "Moved"} {cleanup.summary.successCount} of {cleanup.summary.selectedCount} items{cleanup.summary.action === "moveToTrash" ? " to Trash" : ""}</h2><p className="mt-1 text-sm text-muted">{cleanup.summary.skippedCount} skipped · {cleanup.summary.failureCount} failed · free space changed by {formatBytes(cleanup.summary.actualFreeSpaceChangeBytes ?? 0)}</p>{cleanup.summary.action === "moveToTrash" && <p className="mt-1 text-xs text-muted">Items in Trash still occupy disk space until Trash is emptied.</p>}</div></div>
            <Button onClick={cleanup.reset} variant="ghost"><XCircle size={15} />Dismiss</Button>
          </div>
        </Card>
      )}
      {cleanup.progress && cleanup.status === "running" && (
        <Card className="mt-6 border-sage-400/20 p-5">
          <div className="flex items-center justify-between gap-5"><div><p className="font-semibold">Moving reviewed items to Trash</p><p className="mt-1 max-w-xl truncate text-xs text-muted">{cleanup.progress.currentPath ?? "Finalizing local history…"}</p></div><div className="text-right"><p className="font-semibold tabular-nums">{cleanup.progress.completedItems} / {cleanup.progress.totalItems}</p><Button className="mt-2" onClick={() => void cleanup.cancel()} variant="secondary">Cancel remaining</Button></div></div>
          <div className="mt-4 h-1.5 overflow-hidden rounded-full bg-white/5"><div className="h-full bg-sage-400" style={{ width: `${cleanup.progress.totalItems ? (cleanup.progress.completedItems / cleanup.progress.totalItems) * 100 : 0}%` }} /></div>
        </Card>
      )}

      {findings.length > 0 && (
        <div className="mt-7 flex gap-3">
          <label className="relative flex-1"><Search className="absolute left-3 top-3 text-muted" size={16} /><span className="sr-only">Search findings</span><input className="control pl-10" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search by name or path" /></label>
          <label className="relative w-56"><Filter className="absolute left-3 top-3 text-muted" size={16} /><span className="sr-only">Filter category</span><select className="control pl-10" value={category} onChange={(event) => setCategory(event.target.value as RuleCategory | "all")}><option value="all">All categories</option>{categories.map((item) => <option key={item} value={item}>{categoryLabels[item]}</option>)}</select></label>
        </div>
      )}

      {error ? <Card className="mt-7 border-amber-400/20 p-6" role="alert"><p className="font-semibold">Findings could not be loaded</p><p className="mt-2 text-sm text-muted">{error.message}</p>{summary?.scanId && <Button className="mt-4" variant="secondary" onClick={() => void load(summary.scanId)}>Try again</Button>}</Card> : findings.length === 0 ? (
        <Card className="mt-7 grid min-h-80 place-items-center p-10 text-center">
          <div><FolderSearch className="mx-auto text-muted" size={34} /><h2 className="mt-4 font-semibold">No findings yet</h2><p className="mt-2 text-sm text-muted">Run Quick Scan or Developer Scan to inspect known cache locations.</p></div>
        </Card>
      ) : visible.length === 0 ? (
        <Card className="mt-5 p-8 text-center text-sm text-muted">No findings match the current filters.</Card>
      ) : (
        <VirtualizedList className="mt-5 space-y-3 pb-24" items={visible} itemKey={(finding) => finding.id} estimateSize={() => 210} label="Scan findings" renderItem={(finding) => {
          const size = presentStorageSize(finding.logicalSize, finding.allocatedSize);
          return (
            <Card key={finding.id} className="p-5 shadow-none">
              <div className="flex items-start gap-4">
                <input aria-label={`Select ${finding.displayName}`} checked={selected.has(finding.id)} className="mt-3 size-4 accent-emerald-400" disabled={!finding.cleanupAllowed || cleanupBusy} onChange={() => toggle(finding.id)} type="checkbox" />
                <div className="grid size-10 shrink-0 place-items-center rounded-xl bg-sage-400/10 text-sage-300"><HardDriveDownload size={19} /></div>
                <div className="min-w-0 flex-1"><div className="flex items-center gap-2"><h2 className="font-semibold">{finding.displayName}</h2><span className="rounded-full bg-sage-400/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-sage-100">{finding.risk}</span></div><p className="mt-1 text-sm leading-5 text-muted">{finding.description}</p><p className="mt-3 truncate font-mono text-xs text-muted">{finding.displayPath}</p></div>
                <div className="text-right"><p className="font-semibold tabular-nums">{formatBytes(size.displayedBytes)}</p><p className="mt-1 text-xs text-muted">{size.usesAllocatedSize ? "on disk" : categoryLabels[finding.category]}</p>{size.hasDistinctLogicalSize && <p className="mt-1 whitespace-nowrap text-xs text-muted">{formatBytes(size.logicalBytes)} {finding.category === "container" ? "virtual capacity" : "logical size"}</p>}{size.usesAllocatedSize && <p className="mt-1 text-xs text-muted">{categoryLabels[finding.category]}</p>}<Button className="mt-3" variant="ghost" onClick={() => void commands.revealItem(finding.scanId, finding.id)}><ExternalLink size={14} />Reveal</Button>{finding.guidedAction && <Button className="mt-1" variant="ghost" onClick={() => toggleGuide(finding.id)}><SquareTerminal size={14} />{expandedGuides.has(finding.id) ? "Hide guide" : "Guide"}</Button>}</div>
              </div>
              {finding.guidedAction && expandedGuides.has(finding.id) && <div className="mt-4 rounded-xl border border-sage-400/20 bg-canvas/50 p-4"><p className="text-sm font-semibold">{finding.guidedAction.title}</p><p className="mt-1 text-xs leading-5 text-muted">{finding.guidedAction.explanation}</p><code className="mt-3 block overflow-x-auto rounded-lg border border-line bg-black/20 p-3 text-xs text-sage-100 select-all">{finding.guidedAction.command}</code></div>}
              <div className="mt-4 flex items-center gap-2 border-t border-line pt-3 text-xs text-muted"><ShieldCheck size={13} />{finding.cleanupAllowed ? finding.risk === "careful" ? "Manual selection requires a typed confirmation and a fresh Trash revalidation." : "Eligible for an immutable, revalidated Trash plan." : finding.cleanupBlockReason}</div>
            </Card>
          );
        }} />
      )}

      {selected.size > 0 && cleanup.status !== "review" && (
        <div className="sticky bottom-5 z-20 mt-5 flex items-center justify-between rounded-2xl border border-sage-400/25 bg-panel/95 p-4 shadow-2xl backdrop-blur">
          <div><p className="font-semibold">{selected.size} {selected.size === 1 ? "item" : "items"} selected</p><p className="mt-1 text-xs text-muted">{selectedRisk.safe} safe · {selectedRisk.careful} careful · {formatBytes(selectedBytes)} will be revalidated.</p></div>
          <div className="flex gap-2"><Button disabled={cleanupBusy} onClick={() => createPlan("moveToTrash")}><Trash2 size={16} />{cleanup.status === "planning" ? "Creating plan…" : "Review Trash plan"}</Button>{settings?.permanentDeletionEnabled && selectedRisk.careful === 0 && <Button disabled={cleanupBusy} onClick={() => createPlan("permanentDelete")} variant="destructive"><Trash2 size={16} />Review permanent delete</Button>}</div>
        </div>
      )}

      {cleanup.plan && (cleanup.status === "review" || cleanup.status === "starting") && (
        <CleanupReviewDialog busy={cleanup.status === "starting"} onCancel={cleanup.dismissPlan} onConfirm={(phrase) => void cleanup.executePlan(phrase)} plan={cleanup.plan} />
      )}
    </div>
  );
}
