import { useEffect, useMemo, useState } from "react";
import { ExternalLink, Filter, FolderSearch, HardDriveDownload, Search, ShieldCheck } from "lucide-react";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { commands } from "../ipc/commands";
import type { RuleCategory } from "../ipc/types";
import { formatBytes } from "../lib/utils";
import { useFindingsStore } from "../stores/findingsStore";
import { useScanStore } from "../stores/scanStore";

const categoryLabels: Record<RuleCategory, string> = {
  applicationCache: "Application cache", browserCache: "Browser cache", packageManagerCache: "Package manager",
  buildArtifact: "Build artifact", log: "Logs", installer: "Installers", duplicate: "Duplicates", largeFile: "Large files",
  oldFile: "Old files", container: "Containers", emulator: "Emulators",
};

export function FindingsPage() {
  const { findings, status, error, load } = useFindingsStore();
  const summary = useScanStore((state) => state.summary);
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<RuleCategory | "all">("all");

  useEffect(() => {
    if (summary?.scanId && findings.length === 0 && status === "idle") void load(summary.scanId);
  }, [findings.length, load, status, summary?.scanId]);

  const categories = useMemo(() => Array.from(new Set(findings.map((finding) => finding.category))), [findings]);
  const visible = useMemo(() => findings
    .filter((finding) => category === "all" || finding.category === category)
    .filter((finding) => `${finding.displayName} ${finding.displayPath}`.toLowerCase().includes(query.toLowerCase()))
    .sort((left, right) => (right.allocatedSize ?? right.logicalSize) - (left.allocatedSize ?? left.logicalSize)), [category, findings, query]);
  const total = visible.reduce((sum, finding) => sum + (finding.allocatedSize ?? finding.logicalSize), 0);

  return (
    <div className="mx-auto max-w-6xl px-8 py-8">
      <div className="flex items-end justify-between gap-6">
        <div>
          <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Latest scan</p>
          <h1 className="text-3xl font-semibold tracking-tight">Findings</h1>
          <p className="mt-2 text-sm text-muted">Known, regenerable cache locations with evidence from backend rules.</p>
        </div>
        <div className="text-right"><p className="text-xs text-muted">Visible reclaimable</p><p className="mt-1 text-2xl font-semibold">{formatBytes(total)}</p></div>
      </div>

      {findings.length > 0 && (
        <div className="mt-7 flex gap-3">
          <label className="relative flex-1"><Search className="absolute left-3 top-3 text-muted" size={16} /><span className="sr-only">Search findings</span><input className="control pl-10" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search by name or path" /></label>
          <label className="relative w-56"><Filter className="absolute left-3 top-3 text-muted" size={16} /><span className="sr-only">Filter category</span><select className="control pl-10" value={category} onChange={(event) => setCategory(event.target.value as RuleCategory | "all")}><option value="all">All categories</option>{categories.map((item) => <option key={item} value={item}>{categoryLabels[item]}</option>)}</select></label>
        </div>
      )}

      {error ? <Card className="mt-7 border-amber-400/20 p-6" role="alert">{error.message}</Card> : findings.length === 0 ? (
        <Card className="mt-7 grid min-h-80 place-items-center p-10 text-center">
          <div><FolderSearch className="mx-auto text-muted" size={34} /><h2 className="mt-4 font-semibold">No findings yet</h2><p className="mt-2 text-sm text-muted">Run Quick Scan or Developer Scan to inspect known cache locations.</p></div>
        </Card>
      ) : visible.length === 0 ? (
        <Card className="mt-5 p-8 text-center text-sm text-muted">No findings match the current filters.</Card>
      ) : (
        <div className="mt-5 space-y-3">
          {visible.map((finding) => (
            <Card key={finding.id} className="p-5 shadow-none">
              <div className="flex items-start gap-4">
                <div className="grid size-10 shrink-0 place-items-center rounded-xl bg-sage-400/10 text-sage-300"><HardDriveDownload size={19} /></div>
                <div className="min-w-0 flex-1"><div className="flex items-center gap-2"><h2 className="font-semibold">{finding.displayName}</h2><span className="rounded-full bg-sage-400/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-sage-100">Safe</span></div><p className="mt-1 text-sm leading-5 text-muted">{finding.description}</p><p className="mt-3 truncate font-mono text-xs text-muted">{finding.displayPath}</p></div>
                <div className="text-right"><p className="font-semibold tabular-nums">{formatBytes(finding.allocatedSize ?? finding.logicalSize)}</p><p className="mt-1 text-xs text-muted">{categoryLabels[finding.category]}</p><Button className="mt-3" variant="ghost" onClick={() => void commands.revealItem(finding.scanId, finding.id)}><ExternalLink size={14} />Reveal</Button></div>
              </div>
              <div className="mt-4 flex items-center gap-2 border-t border-line pt-3 text-xs text-muted"><ShieldCheck size={13} />{finding.cleanupBlockReason}</div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}

