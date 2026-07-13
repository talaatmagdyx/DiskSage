import { useEffect, useState } from "react";
import { CheckCircle2, History, Trash2, TriangleAlert } from "lucide-react";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type { CleanupSummary, CommandError } from "../ipc/types";
import { formatBytes } from "../lib/utils";

export function HistoryPage() {
  const [entries, setEntries] = useState<CleanupSummary[]>([]);
  const [error, setError] = useState<CommandError | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void commands.getCleanupHistory().then(setEntries).catch((value) => setError(normalizeCommandError(value))).finally(() => setLoading(false));
  }, []);

  const clear = async () => {
    try {
      await commands.clearCleanupHistory();
      setEntries([]);
    } catch (value) {
      setError(normalizeCommandError(value));
    }
  };

  return (
    <div className="mx-auto max-w-5xl px-8 py-8">
      <div className="flex items-end justify-between gap-5">
        <div><p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Local audit trail</p><h1 className="text-3xl font-semibold">Cleanup history</h1><p className="mt-2 text-sm text-muted">Stored only on this device, including skips and per-item failures.</p></div>
        {entries.length > 0 && <Button onClick={() => void clear()} variant="secondary"><Trash2 size={15} />Clear history</Button>}
      </div>
      {error ? <Card className="mt-7 border-amber-400/20 p-5" role="alert">{error.message}</Card> : loading ? <Card className="mt-7 p-8 text-sm text-muted">Loading local history…</Card> : entries.length === 0 ? (
        <Card className="mt-7 grid min-h-72 place-items-center p-10 text-center"><div><History className="mx-auto text-muted" size={34} /><h2 className="mt-4 font-semibold">No cleanup history</h2><p className="mt-2 text-sm text-muted">Completed cleanup plans will appear here.</p></div></Card>
      ) : <div className="mt-7 space-y-3">{entries.map((entry) => (
        <Card className="p-5" key={entry.operationId}>
          <div className="flex items-start justify-between gap-5"><div className="flex gap-3">{entry.failureCount > 0 || entry.skippedCount > 0 ? <TriangleAlert className="text-amber-300" size={20} /> : <CheckCircle2 className="text-sage-300" size={20} />}<div><h2 className="font-semibold">{entry.action === "permanentDelete" ? "Permanently deleted" : "Moved"} {entry.successCount} of {entry.selectedCount} items{entry.action === "moveToTrash" ? " to Trash" : ""}</h2><p className="mt-1 text-xs text-muted">{new Date(entry.completedAt).toLocaleString()} · {entry.failureCount} failed · {entry.skippedCount} skipped</p></div></div><div className="text-right"><p className="font-semibold">{formatBytes(entry.expectedBytes)}</p><p className="mt-1 text-xs text-muted">selected size</p></div></div>
          {(entry.failureCount > 0 || entry.skippedCount > 0) && <div className="mt-4 space-y-2 border-t border-line pt-3">{entry.items.filter((item) => item.status === "failed" || item.status === "skipped").map((item) => <p className="truncate text-xs text-muted" key={item.findingId}>{item.displayPath}: {item.error?.message ?? item.status}</p>)}</div>}
        </Card>
      ))}</div>}
    </div>
  );
}
