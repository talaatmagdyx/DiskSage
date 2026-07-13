import {
  CircleGauge,
  Files,
  History,
  ScanSearch,
  Settings,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";
import { cn } from "../../lib/utils";
import { useScanEvents } from "../../hooks/useScanEvents";

const links = [
  { to: "/", label: "Overview", icon: CircleGauge, end: true },
  { to: "/scan", label: "Scan", icon: ScanSearch },
  { to: "/cleanup", label: "Findings", icon: Sparkles },
  { to: "/duplicates", label: "Duplicates", icon: Files },
  { to: "/history", label: "History", icon: History },
];

export function AppShell() {
  useScanEvents();
  return (
    <div className="grid min-h-full grid-cols-[230px_1fr]">
      <aside className="flex h-screen flex-col border-r border-line bg-[#07100e]/90 px-4 py-5">
        <div className="mb-8 flex items-center gap-3 px-2">
          <div className="grid size-10 place-items-center rounded-xl border border-sage-400/30 bg-sage-400/10 text-sage-300">
            <ShieldCheck aria-hidden="true" size={22} />
          </div>
          <div>
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
              {label}
            </NavLink>
          ))}
        </nav>

        <div className="mt-auto">
          <div className="mb-3 rounded-xl border border-sage-400/15 bg-sage-400/[0.06] p-3">
            <p className="flex items-center gap-2 text-xs font-semibold text-sage-100">
              <ShieldCheck aria-hidden="true" size={14} /> No destructive actions
            </p>
            <p className="mt-1 text-xs leading-relaxed text-muted">This foundation build only reads disk metadata.</p>
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
            <Settings aria-hidden="true" size={18} /> Settings
          </NavLink>
        </div>
      </aside>
      <main className="h-screen overflow-y-auto">
        <Outlet />
      </main>
    </div>
  );
}
