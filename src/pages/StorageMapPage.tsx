import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen, HardDrive, LoaderCircle, Map, ShieldCheck, TriangleAlert } from "lucide-react";
import { StorageAccuracyPanel } from "../components/storage/StorageAccuracyPanel";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { formatBytes } from "../lib/utils";
import { useStorageMapStore } from "../stores/storageMapStore";

export function StorageMapPage() {
  const store = useStorageMapStore();
  const scanning = store.status === "scanning";
  const chooseFolder = async () => {
    const selected = await open({ directory: true, multiple: false, title: "Choose a folder to analyze" });
    if (typeof selected === "string") await store.scan(selected);
  };
  const largest = Math.max(...(store.report?.entries.map((entry) => entry.allocatedSize) ?? [1]), 1);

  return (
    <div className="mx-auto max-w-6xl px-8 py-8 pb-14">
      <div className="flex items-end justify-between gap-6">
        <div><p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Read-only folder analysis</p><h1 className="text-3xl font-semibold tracking-tight">Storage Map</h1><p className="mt-2 max-w-2xl text-sm leading-6 text-muted">Compare the folders using space without classifying them as safe to remove. Analysis stays inside your home folder, never follows symlinks, and crosses no filesystem boundary.</p></div>
        <div className="flex gap-3"><Button disabled={scanning} onClick={() => void store.scan()} variant="secondary"><HardDrive size={16} />Analyze Home</Button><Button disabled={scanning} onClick={() => void chooseFolder()}><FolderOpen size={16} />Choose folder</Button></div>
      </div>

      {store.error && <Card className="mt-6 flex gap-3 border-amber-400/20 p-5" role="alert"><TriangleAlert className="mt-0.5 shrink-0 text-amber-300" size={19} /><div><p className="font-semibold">Folder could not be analyzed</p><p className="mt-1 text-sm text-muted">{store.error.message}</p></div></Card>}
      {scanning && <Card className="mt-7 grid min-h-56 place-items-center text-center"><div><LoaderCircle className="mx-auto animate-spin text-sage-300" size={34} /><h2 className="mt-4 font-semibold">Building a bounded storage map</h2><p className="mt-2 text-sm text-muted">Up to 150,000 entries or 15 seconds; partial results are labeled.</p></div></Card>}

      {!scanning && store.report && (
        <>
          <Card className="mt-7 p-5">
            <div className="flex flex-wrap items-start justify-between gap-4 border-b border-line pb-5">
              <div><div className="flex items-center gap-2"><span className="rounded-full bg-sage-400/10 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wide text-sage-200">Read only</span>{store.report.truncated && <span className="rounded-full bg-amber-400/10 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wide text-amber-200">Partial result</span>}</div><p className="mt-3 font-mono text-sm">{store.report.displayRoot}</p><p className="mt-1 text-xs text-muted">{store.report.filesScanned.toLocaleString()} files · {store.report.directoriesScanned.toLocaleString()} folders · {(store.report.elapsedMs / 1000).toFixed(1)} seconds</p></div>
              <div className="flex gap-8 text-right"><div><p className="text-xs text-muted">Logical</p><p className="mt-1 text-xl font-semibold">{formatBytes(store.report.logicalSize)}</p></div><div><p className="text-xs text-muted">On disk</p><p className="mt-1 text-xl font-semibold">{formatBytes(store.report.allocatedSize)}</p></div></div>
            </div>
            {store.report.permissionDeniedCount > 0 && <p className="mt-4 flex items-center gap-2 text-xs text-amber-200"><TriangleAlert size={14} />{store.report.permissionDeniedCount} protected locations could not be measured. Results may be smaller than Finder.</p>}
            <div className="mt-5 space-y-3">
              {store.report.entries.map((entry) => (
                <div className="rounded-xl border border-line bg-white/[0.02] p-4" key={entry.id}>
                  <div className="flex items-center justify-between gap-5"><div className="min-w-0"><p className="truncate font-semibold">{entry.name}</p><p className="mt-1 truncate font-mono text-xs text-muted">{entry.displayPath}</p></div><div className="shrink-0 text-right"><p className="font-semibold tabular-nums">{formatBytes(entry.allocatedSize)}</p><p className="text-[11px] text-muted">{formatBytes(entry.logicalSize)} logical</p></div></div>
                  <div className="mt-3 h-2 overflow-hidden rounded-full bg-white/5"><div className="h-full rounded-full bg-sage-400" style={{ width: `${Math.max(2, (entry.allocatedSize / largest) * 100)}%` }} /></div>
                  {(entry.truncated || entry.permissionDeniedCount > 0) && <p className="mt-2 text-[11px] text-amber-200">Partial measurement · {entry.permissionDeniedCount} access errors</p>}
                </div>
              ))}
              {store.report.entries.length === 0 && <div className="grid min-h-40 place-items-center text-center"><div><Map className="mx-auto text-muted" size={30} /><p className="mt-3 font-semibold">This folder has no measurable direct children</p></div></div>}
            </div>
            <p className="mt-5 flex items-start gap-2 border-t border-line pt-4 text-xs leading-5 text-muted"><ShieldCheck className="mt-0.5 shrink-0" size={14} />{store.report.note}</p>
          </Card>
          <div className="mt-6"><StorageAccuracyPanel /></div>
        </>
      )}

      {!scanning && !store.report && <Card className="mt-7 grid min-h-60 place-items-center text-center"><div><Map className="mx-auto text-sage-300" size={36} /><h2 className="mt-4 font-semibold">Choose what to explain</h2><p className="mx-auto mt-2 max-w-md text-sm leading-6 text-muted">Start with Home for the broadest view, or choose a smaller folder for a faster and more focused map.</p></div></Card>}
      {!store.report && <div className="mt-6"><StorageAccuracyPanel /></div>}
    </div>
  );
}
