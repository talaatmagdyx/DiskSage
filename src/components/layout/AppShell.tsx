import {
  CircleGauge,
  AppWindow,
  Files,
  History,
  ScanSearch,
  Settings,
  Info,
  ShieldCheck,
 Sparkles,
  Keyboard,
  Map,
 X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { cn } from "../../lib/utils";
import { useScanEvents } from "../../hooks/useScanEvents";
import { useDuplicateEvents } from "../../hooks/useDuplicateEvents";
import { useScanStore } from "../../stores/scanStore";
import { useDuplicateStore } from "../../stores/duplicateStore";
import { useCleanupStore } from "../../stores/cleanupStore";
import { toast } from "../../stores/toastStore";
import { useModalFocus } from "../../hooks/useModalFocus";

const links = [
  { to: "/", label: "Overview", icon: CircleGauge, end: true },
  { to: "/scan", label: "Scan", icon: ScanSearch },
  { to: "/cleanup", label: "Findings", icon: Sparkles },
  { to: "/duplicates", label: "Duplicates", icon: Files },
  { to: "/applications", label: "Applications", icon: AppWindow },
  { to: "/storage-map", label: "Storage Map", icon: Map },
  { to: "/history", label: "History", icon: History },
];

export function AppShell() {
 useScanEvents();
 useDuplicateEvents();
 const navigate = useNavigate();
 const [shortcutsOpen, setShortcutsOpen] = useState(false);
 const shortcutModal = useModalFocus(() => setShortcutsOpen(false), shortcutsOpen);
 const scanSummary = useScanStore((state) => state.summary);
 const scanError = useScanStore((state) => state.error);
 const duplicateSummary = useDuplicateStore((state) => state.summary);
 const duplicateError = useDuplicateStore((state) => state.error);
 const cleanupSummary = useCleanupStore((state) => state.summary);
 const cleanupError = useCleanupStore((state) => state.error);
 const announced = useRef(new Set<string>());

 useEffect(() => {
  const onKeyDown = (event: KeyboardEvent) => {
   const target = event.target as HTMLElement | null;
   const typing = target?.matches("input, textarea, select, [contenteditable='true']");
   if (event.key === "Escape") setShortcutsOpen(false);
   if (!typing && event.key === "?") { event.preventDefault(); setShortcutsOpen(true); }
   if (!(event.metaKey || event.ctrlKey) || event.altKey) return;
   const destinations: Record<string, string> = { "1": "/", "2": "/scan", "3": "/cleanup", "4": "/duplicates", "5": "/applications", "6": "/history", ",": "/settings" };
   const destination = destinations[event.key];
   if (destination) { event.preventDefault(); navigate(destination); }
  };
  window.addEventListener("keydown", onKeyDown);
  return () => window.removeEventListener("keydown", onKeyDown);
 }, [navigate]);

 useEffect(() => {
  if (!scanSummary || announced.current.has(`scan-${scanSummary.scanId}-${scanSummary.phase}`)) return;
  if (["completed", "cancelled", "failed"].includes(scanSummary.phase)) {
   announced.current.add(`scan-${scanSummary.scanId}-${scanSummary.phase}`);
   toast({ tone: scanSummary.phase === "completed" ? "success" : "warning", title: scanSummary.phase === "completed" ? "Scan complete" : scanSummary.phase === "cancelled" ? "Scan cancelled" : "Scan stopped", message: `${scanSummary.findingsCount} findings saved locally.` });
  }
 }, [scanSummary]);
 useEffect(() => { if (scanError && !announced.current.has(`scan-error-${scanError.code}`)) { announced.current.add(`scan-error-${scanError.code}`); toast({ tone: "error", title: "Scan needs attention", message: scanError.message }); } }, [scanError]);
 useEffect(() => {
  if (!duplicateSummary || announced.current.has(`duplicate-${duplicateSummary.scanId}-${duplicateSummary.phase}`)) return;
  if (["completed", "cancelled", "failed"].includes(duplicateSummary.phase)) {
   announced.current.add(`duplicate-${duplicateSummary.scanId}-${duplicateSummary.phase}`);
   toast({ tone: duplicateSummary.phase === "completed" ? "success" : "warning", title: duplicateSummary.phase === "completed" ? "Duplicate scan complete" : "Duplicate scan stopped", message: `${duplicateSummary.groupsFound} verified groups found.` });
  }
 }, [duplicateSummary]);
 useEffect(() => { if (duplicateError && !announced.current.has(`duplicate-error-${duplicateError.code}`)) { announced.current.add(`duplicate-error-${duplicateError.code}`); toast({ tone: "error", title: "Duplicate scan needs attention", message: duplicateError.message }); } }, [duplicateError]);
 useEffect(() => { if (cleanupSummary && !announced.current.has(`cleanup-${cleanupSummary.operationId}`)) { announced.current.add(`cleanup-${cleanupSummary.operationId}`); toast({ tone: cleanupSummary.failureCount ? "warning" : "success", title: "Cleanup finished", message: `${cleanupSummary.successCount} succeeded, ${cleanupSummary.skippedCount} skipped, ${cleanupSummary.failureCount} failed.` }); } }, [cleanupSummary]);
 useEffect(() => { if (cleanupError && !announced.current.has(`cleanup-error-${cleanupError.code}`)) { announced.current.add(`cleanup-error-${cleanupError.code}`); toast({ tone: "error", title: "Cleanup stopped safely", message: cleanupError.message }); } }, [cleanupError]);
 return (
  <div className="grid min-h-full grid-cols-[76px_1fr] lg:grid-cols-[230px_1fr]">
   <aside className="flex h-screen flex-col border-r border-line bg-panel/90 px-3 py-5 lg:px-4">
        <div className="mb-8 flex items-center gap-3 px-2">
          <div className="grid size-10 place-items-center rounded-xl border border-sage-400/30 bg-sage-400/10 text-sage-300">
            <ShieldCheck aria-hidden="true" size={22} />
          </div>
     <div className="hidden lg:block">
            <p className="text-base font-bold tracking-tight">DiskSage</p>
            <p className="text-[11px] uppercase tracking-[0.16em] text-muted">Local by design</p>
          </div>
        </div>

        <nav aria-label="Primary" className="space-y-1">
          {links.map(({ to, label, icon: Icon, end }) => (
            <NavLink
              key={to}
              to={to}
              end={end}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium text-muted transition-colors hover:bg-white/5 hover:text-ink",
                  isActive && "bg-sage-400/10 text-sage-100",
                )
              }
            >
              <Icon aria-hidden="true" size={18} />
       <span className="hidden lg:inline">{label}</span>
            </NavLink>
          ))}
        </nav>

        <div className="mt-auto">
     <div className="mb-3 hidden rounded-xl border border-sage-400/15 bg-sage-400/[0.06] p-3 lg:block">
            <p className="flex items-center gap-2 text-xs font-semibold text-sage-100">
              <ShieldCheck aria-hidden="true" size={14} /> Trash-first cleanup
            </p>
            <p className="mt-1 text-xs leading-relaxed text-muted">Every item is planned, reviewed, and revalidated before it moves.</p>
          </div>
     <NavLink
            to="/settings"
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium text-muted hover:bg-white/5 hover:text-ink",
                isActive && "bg-white/5 text-ink",
              )
            }
          >
      <Settings aria-hidden="true" size={18} /> <span className="hidden lg:inline">Settings</span>
     </NavLink>
     <NavLink
            to="/about"
            className={({ isActive }) =>
              cn(
                "mt-1 flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium text-muted hover:bg-white/5 hover:text-ink",
                isActive && "bg-white/5 text-ink",
              )
            }
          >
      <Info aria-hidden="true" size={18} /> <span className="hidden lg:inline">About</span>
     </NavLink>
     <button className="mt-1 flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium text-muted hover:bg-white/5 hover:text-ink" onClick={() => setShortcutsOpen(true)} aria-label="Keyboard shortcuts">
      <Keyboard aria-hidden="true" size={18} /><span className="hidden lg:inline">Shortcuts</span>
     </button>
        </div>
      </aside>
   <main className="h-screen overflow-y-auto">
        <Outlet />
   </main>
   {shortcutsOpen && <div className="fixed inset-0 z-50 grid place-items-center bg-black/60 p-6" role="presentation" onMouseDown={(event) => { if (event.currentTarget === event.target) setShortcutsOpen(false); }}><div ref={shortcutModal.ref as React.RefObject<HTMLDivElement>} onKeyDown={shortcutModal.onKeyDown} className="w-full max-w-md rounded-2xl border border-line bg-panel p-6 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="shortcut-title"><div className="flex items-center justify-between"><h2 id="shortcut-title" className="text-lg font-semibold">Keyboard shortcuts</h2><button className="rounded-lg p-2 text-muted hover:bg-white/5 hover:text-ink" onClick={() => setShortcutsOpen(false)} aria-label="Close keyboard shortcuts"><X size={18} /></button></div><dl className="mt-5 grid grid-cols-[1fr_auto] gap-x-5 gap-y-3 text-sm"><dt>Overview</dt><dd><kbd>⌘/Ctrl 1</kbd></dd><dt>Scan</dt><dd><kbd>⌘/Ctrl 2</kbd></dd><dt>Findings</dt><dd><kbd>⌘/Ctrl 3</kbd></dd><dt>Duplicates</dt><dd><kbd>⌘/Ctrl 4</kbd></dd><dt>Applications</dt><dd><kbd>⌘/Ctrl 5</kbd></dd><dt>History</dt><dd><kbd>⌘/Ctrl 6</kbd></dd><dt>Settings</dt><dd><kbd>⌘/Ctrl ,</kbd></dd><dt>Show this help</dt><dd><kbd>?</kbd></dd><dt>Close a dialog</dt><dd><kbd>Esc</kbd></dd></dl></div></div>}
  </div>
  );
}
