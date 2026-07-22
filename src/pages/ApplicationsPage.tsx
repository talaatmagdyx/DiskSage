import { useEffect, useMemo, useState } from "react";
import {
  AppWindow,
  CheckCircle2,
  ExternalLink,
  Info,
  LoaderCircle,
  PackageX,
  RefreshCw,
  Search,
  ShieldCheck,
  TriangleAlert,
} from "lucide-react";
import { ApplicationUninstallDialog } from "../components/applications/ApplicationUninstallDialog";
import { UninstallModeDialog } from "../components/applications/UninstallModeDialog";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import type { ApplicationScope, ApplicationUninstallResult, InstalledApplication } from "../ipc/types";
import { formatBytes } from "../lib/utils";
import { useApplicationStore } from "../stores/applicationStore";
import { toast } from "../stores/toastStore";

type SortOrder =
  | "lastUsedOldest"
  | "lastUsedNewest"
  | "sizeLargest"
  | "sizeSmallest"
  | "nameAscending"
  | "nameDescending";
type ScopeFilter = "all" | ApplicationScope;

export function ApplicationsPage() {
  const store = useApplicationStore();
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SortOrder>("lastUsedOldest");
  const [scope, setScope] = useState<ScopeFilter>("all");
  const [uninstallChoice, setUninstallChoice] = useState<InstalledApplication | null>(null);

  useEffect(() => {
    if (store.status === "idle") void store.scan();
  }, [store]);

  useEffect(() => {
    if (!store.error) return;
    toast({ tone: "error", title: "Application action needs attention", message: store.error.message });
  }, [store.error]);

  const applications = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return store.applications
      .filter((application) => scope === "all" || application.scope === scope)
      .filter((application) =>
        !normalized
        || application.name.toLowerCase().includes(normalized)
        || application.bundleId?.toLowerCase().includes(normalized)
        || application.displayPath.toLowerCase().includes(normalized),
      )
      .sort((left, right) => compareApplications(left, right, sort));
  }, [query, scope, sort, store.applications]);

  const scanning = store.status === "scanning";
  const totalBytes = applications.reduce(
    (total, application) => total + (application.allocatedSize ?? application.logicalSize),
    0,
  );

  return (
    <div className="mx-auto max-w-6xl px-8 py-8">
      <div className="flex items-end justify-between gap-6">
        <div>
          <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Installed software inventory</p>
          <h1 className="text-3xl font-semibold tracking-tight">Applications</h1>
          <p className="mt-2 max-w-2xl text-sm leading-6 text-muted">
            Review installed app bundles by size and last-used date. Last-used metadata comes from macOS and may be unavailable for some apps.
          </p>
        </div>
        <div className="flex items-end gap-5">
          {store.applications.length > 0 && <div className="text-right"><p className="text-xs text-muted">Visible application size</p><p className="mt-1 text-2xl font-semibold">{formatBytes(totalBytes)}</p></div>}
          <Button disabled={scanning || store.status === "uninstalling"} onClick={() => void store.scan()} variant="secondary">
            {scanning ? <LoaderCircle className="animate-spin" size={16} /> : <RefreshCw size={16} />}
            {scanning ? "Scanning…" : "Scan again"}
          </Button>
        </div>
      </div>

      {store.error && (
        <Card className="mt-6 flex items-start gap-3 border-amber-400/20 p-5" role="alert">
          <TriangleAlert className="mt-0.5 shrink-0 text-amber-300" size={19} />
          <div className="min-w-0 flex-1">
            <p className="font-semibold">Application action needs attention</p>
            <p className="mt-1 text-sm text-muted">{store.error.message}</p>
            {store.retryUninstallContext && store.status === "ready" && (
              <Button className="mt-3" onClick={() => void store.retryUninstall()} variant="secondary">
                <RefreshCw size={14} />Review again
              </Button>
            )}
          </div>
        </Card>
      )}
      {store.result && (
        <Card className={`mt-6 flex items-start justify-between gap-4 p-5 ${store.result.relatedItemsFailed ? "border-amber-400/25" : "border-sage-400/20"}`}>
          <div className="flex gap-3">
            <CheckCircle2 className={`mt-0.5 shrink-0 ${store.result.relatedItemsFailed ? "text-amber-300" : "text-sage-300"}`} size={19} />
            <div>
              <p className="font-semibold">{store.result.name} moved to Trash</p>
              <p className="mt-1 text-sm text-muted">{resultMessage(store.result)}</p>
              {store.result.failedItems.length > 0 && (
                <ul className="mt-3 space-y-2">
                  {store.result.failedItems.map((failure) => (
                    <li className="rounded-lg border border-amber-400/15 bg-amber-400/5 p-3" key={failure.displayPath}>
                      <p className="font-mono text-xs text-amber-200">{failure.displayPath}</p>
                      <p className="mt-1 text-xs leading-5 text-muted">{failure.message}</p>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>
          <button className="text-xs text-muted hover:text-ink" onClick={store.clearResult}>Dismiss</button>
        </Card>
      )}

      <Card className="mt-7 p-5">
        <div className="grid grid-cols-[1fr_220px_180px] gap-3">
          <label className="relative"><span className="sr-only">Search applications</span><Search className="absolute left-3 top-3 text-muted" size={17} /><input className="control pl-10" onChange={(event) => setQuery(event.target.value)} placeholder="Search by app, bundle ID, or path" value={query} /></label>
          <label><span className="sr-only">Sort applications</span><select className="control" onChange={(event) => setSort(event.target.value as SortOrder)} value={sort}><option value="lastUsedOldest">Least recently used</option><option value="lastUsedNewest">Most recently used</option><option value="sizeLargest">Size: largest first</option><option value="sizeSmallest">Size: smallest first</option><option value="nameAscending">Name: A–Z</option><option value="nameDescending">Name: Z–A</option></select></label>
          <label><span className="sr-only">Filter by scope</span><select className="control" onChange={(event) => setScope(event.target.value as ScopeFilter)} value={scope}><option value="all">All scanned locations</option><option value="user">User apps</option><option value="shared">Shared apps</option>{store.includeSystemApps && <option value="system">System apps</option>}</select></label>
        </div>
        <div className="mt-4 flex items-center justify-between gap-6 border-t border-line pt-4">
          <div className="flex items-start gap-2 text-xs text-muted">
            <Info className="mt-0.5 shrink-0" size={14} />
            <span>Size is measured from the bundle currently on disk. System apps are protected and can never be selected for uninstall.</span>
          </div>
          <label className="flex shrink-0 cursor-pointer items-center gap-3 rounded-lg border border-line bg-white/[0.02] px-3 py-2 text-sm">
            <input
              checked={!store.includeSystemApps}
              disabled={scanning || store.status === "uninstalling"}
              onChange={(event) => {
                const includeSystemApps = !event.target.checked;
                if (!includeSystemApps && scope === "system") setScope("all");
                void store.scan(includeSystemApps);
              }}
              type="checkbox"
            />
            <span>Exclude system apps</span>
          </label>
        </div>
      </Card>

      {scanning && store.applications.length === 0 ? (
        <Card className="mt-6 grid min-h-64 place-items-center text-center"><div><LoaderCircle className="mx-auto animate-spin text-sage-300" size={34} /><h2 className="mt-4 font-semibold">Scanning installed applications</h2><p className="mt-2 text-sm text-muted">Measuring bundles and reading local macOS metadata…</p></div></Card>
      ) : applications.length === 0 ? (
        <Card className="mt-6 grid min-h-56 place-items-center text-center"><div><AppWindow className="mx-auto text-muted" size={34} /><h2 className="mt-4 font-semibold">No matching applications</h2><p className="mt-2 text-sm text-muted">Adjust the search or scan installed applications again.</p></div></Card>
      ) : (
        <div className="mt-6 space-y-3 pb-12">
          {applications.map((application) => (
            <ApplicationRow
              application={application}
              busy={["planning", "review", "uninstalling"].includes(store.status)}
              operation={store.activeApplicationId === application.id ? store.status : null}
              key={application.id}
              onReveal={() => void store.reveal(application.id)}
              onUninstall={() => setUninstallChoice(application)}
            />
          ))}
        </div>
      )}

      {store.plan && (store.status === "review" || store.status === "uninstalling") && (
        <ApplicationUninstallDialog
          busy={store.status === "uninstalling"}
          onCancel={store.dismissPlan}
          onConfirm={(selectedRelatedItemIds, typedConfirmation) => void store.executePlan(selectedRelatedItemIds, typedConfirmation)}
          plan={store.plan}
        />
      )}
      {uninstallChoice && store.status === "ready" && (
        <UninstallModeDialog
          application={uninstallChoice}
          onCancel={() => setUninstallChoice(null)}
          onChoose={(mode) => {
            const applicationId = uninstallChoice.id;
            setUninstallChoice(null);
            void store.reviewUninstall(applicationId, mode);
          }}
        />
      )}
    </div>
  );
}

function ApplicationRow({ application, busy, operation, onReveal, onUninstall }: { application: InstalledApplication; busy: boolean; operation: string | null; onReveal: () => void; onUninstall: () => void }) {
  const bytes = application.allocatedSize ?? application.logicalSize;
  return (
    <Card className="flex items-center gap-4 p-5 shadow-none">
      <div className="grid size-11 shrink-0 place-items-center rounded-xl bg-sage-400/10 text-sage-300"><AppWindow size={21} /></div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2"><h2 className="truncate font-semibold">{application.name}</h2><span className="rounded-full bg-white/5 px-2 py-0.5 text-[10px] uppercase tracking-wide text-muted">{application.scope}</span></div>
        <p className="mt-1 truncate font-mono text-xs text-muted">{application.displayPath}</p>
        <p className="mt-2 text-xs text-muted">{application.version ? `Version ${application.version} · ` : ""}{lastUsedLabel(application.lastUsedAt)}</p>
        {!application.uninstallAllowed && <p className="mt-2 flex items-center gap-1.5 text-xs text-amber-200"><ShieldCheck size={13} />{application.uninstallBlockReason}</p>}
      </div>
      <div className="w-28 text-right"><p className="font-semibold tabular-nums">{formatBytes(bytes)}</p><p className="mt-1 text-[11px] text-muted">on disk</p></div>
      <Button aria-label={`Reveal ${application.name}`} onClick={onReveal} variant="ghost"><ExternalLink size={14} />Reveal</Button>
      <Button aria-label={`Uninstall ${application.name}`} disabled={busy || !application.uninstallAllowed} onClick={onUninstall} variant="secondary">{operation ? <LoaderCircle className="animate-spin" size={15} /> : <PackageX size={15} />}{operation === "planning" ? "Preparing…" : operation === "uninstalling" ? "Moving…" : "Uninstall"}</Button>
    </Card>
  );
}

function lastUsedLabel(value?: string) {
  if (!value) return "Last used unavailable";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Last used unavailable";
  return `Last used ${date.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" })}`;
}

function compareApplications(left: InstalledApplication, right: InstalledApplication, sort: SortOrder) {
  if (sort === "nameAscending") return left.name.localeCompare(right.name);
  if (sort === "nameDescending") return right.name.localeCompare(left.name);
  if (sort === "sizeLargest") return (right.allocatedSize ?? right.logicalSize) - (left.allocatedSize ?? left.logicalSize);
  if (sort === "sizeSmallest") return (left.allocatedSize ?? left.logicalSize) - (right.allocatedSize ?? right.logicalSize);
  const leftTime = left.lastUsedAt ? new Date(left.lastUsedAt).getTime() : Number.NEGATIVE_INFINITY;
  const rightTime = right.lastUsedAt ? new Date(right.lastUsedAt).getTime() : Number.NEGATIVE_INFINITY;
  return sort === "lastUsedNewest" ? rightTime - leftTime : leftTime - rightTime;
}

function resultMessage(result: ApplicationUninstallResult) {
  if (result.mode === "appOnly") return "The app bundle moved; related preferences and user data were left untouched.";
  if (result.relatedItemsFailed) return `${result.relatedItemsMoved} related items moved; ${result.relatedItemsFailed} could not be moved and remain in place.`;
  if (result.mode === "deepCleanup") return `${result.relatedItemsMoved} explicitly selected related items moved with the app.`;
  return `${result.relatedItemsMoved} related ${result.relatedItemsMoved === 1 ? "item" : "items"} moved with the app. Shared and personal folders remained untouched.`;
}
