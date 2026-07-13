import { useEffect } from "react";
import { Database, HardDrive, LockKeyhole, RefreshCw, ShieldCheck } from "lucide-react";
import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import { Link } from "react-router-dom";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { formatBytes } from "../lib/utils";
import { useDiskStore } from "../stores/diskStore";

function DiskChart({ used, available }: { used: number; available: number }) {
  const data = [
    { name: "Used", value: used, color: "#219d72" },
    { name: "Available", value: available, color: "#1a302a" },
  ];

  return (
    <div className="relative h-40 w-40" role="img" aria-label={`${formatBytes(used)} used and ${formatBytes(available)} available`}>
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie data={data} dataKey="value" innerRadius={55} outerRadius={72} strokeWidth={0}>
            {data.map((entry) => <Cell key={entry.name} fill={entry.color} />)}
          </Pie>
          <Tooltip
            formatter={(value) => formatBytes(typeof value === "number" ? value : Number(value ?? 0))}
            contentStyle={{ background: "#0d1c18", border: "1px solid #233a33", borderRadius: 12 }}
          />
        </PieChart>
      </ResponsiveContainer>
      <div className="pointer-events-none absolute inset-0 grid place-content-center text-center">
        <span className="text-xs text-muted">Total</span>
        <strong className="text-lg">{formatBytes(used + available)}</strong>
      </div>
    </div>
  );
}

export function DashboardPage() {
  const { disks, status, error, refresh } = useDiskStore();

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const primary = disks[0];

  return (
    <div className="mx-auto max-w-6xl px-8 py-8">
      <header className="mb-8 flex items-start justify-between gap-6">
        <div>
          <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Storage overview</p>
          <h1 className="text-3xl font-semibold tracking-tight">Your disk, explained clearly.</h1>
          <p className="mt-2 max-w-2xl text-sm leading-6 text-muted">
            DiskSage reads storage metadata locally. No filenames, paths, hashes, or results leave this device.
          </p>
        </div>
        <Button variant="secondary" onClick={() => void refresh()} disabled={status === "loading"}>
          <RefreshCw aria-hidden="true" className={status === "loading" ? "animate-spin" : ""} size={16} />
          Refresh
        </Button>
      </header>

      {status === "loading" && disks.length === 0 ? (
        <Card className="grid min-h-72 place-items-center p-8" aria-live="polite">
          <div className="text-center text-muted"><RefreshCw className="mx-auto mb-3 animate-spin" /> Reading mounted disks…</div>
        </Card>
      ) : error ? (
        <Card className="border-amber-400/20 p-8" role="alert">
          <p className="font-semibold text-amber-100">Disk information is unavailable</p>
          <p className="mt-2 text-sm text-muted">{error.message}</p>
          <Button className="mt-5" variant="secondary" onClick={() => void refresh()}>Try again</Button>
        </Card>
      ) : !primary ? (
        <Card className="p-8 text-center">
          <HardDrive className="mx-auto mb-3 text-muted" aria-hidden="true" />
          <p className="font-semibold">No accessible disk found</p>
          <p className="mt-2 text-sm text-muted">Check system permissions, then refresh.</p>
        </Card>
      ) : (
        <>
          <div className="grid grid-cols-[1.5fr_1fr] gap-5">
            <Card className="p-6">
              <div className="flex items-center justify-between gap-6">
                <div className="min-w-0 flex-1">
                  <div className="mb-6 flex items-center gap-3">
                    <div className="grid size-10 place-items-center rounded-xl bg-white/5 text-sage-300">
                      <HardDrive aria-hidden="true" size={21} />
                    </div>
                    <div className="min-w-0">
                      <h2 className="truncate font-semibold">{primary.name || "Local disk"}</h2>
                      <p className="truncate font-mono text-xs text-muted">{primary.mountPath} · {primary.fileSystem || "Unknown filesystem"}</p>
                    </div>
                  </div>
                  <div className="grid grid-cols-3 gap-3">
                    <Metric label="Used" value={formatBytes(primary.usedBytes)} />
                    <Metric label="Available" value={formatBytes(primary.availableBytes)} />
                    <Metric label="Usage" value={`${primary.percentageUsed.toFixed(1)}%`} />
                  </div>
                </div>
                <DiskChart used={primary.usedBytes} available={primary.availableBytes} />
              </div>
            </Card>

            <Card className="flex flex-col justify-between p-6">
              <div>
                <span className="inline-flex items-center gap-2 rounded-full bg-sage-400/10 px-3 py-1 text-xs font-semibold text-sage-100">
                  <ShieldCheck aria-hidden="true" size={14} /> Foundation ready
                </span>
                <h2 className="mt-5 text-xl font-semibold">Targeted scans are ready.</h2>
                <p className="mt-2 text-sm leading-6 text-muted">
                  Quick and Developer scans inspect ten backend-owned cache rules with cancellation, progress, and local findings persistence.
                </p>
                <Link to="/scan" className="mt-5 inline-flex rounded-xl bg-sage-400 px-4 py-2.5 text-sm font-semibold text-sage-900">Choose a scan</Link>
              </div>
              <div className="mt-6 flex items-center gap-2 text-xs text-muted">
                <LockKeyhole aria-hidden="true" size={14} /> Destructive commands are not registered.
              </div>
            </Card>
          </div>

          <section className="mt-7" aria-labelledby="mounted-disks">
            <div className="mb-3 flex items-center justify-between">
              <h2 id="mounted-disks" className="font-semibold">Mounted disks</h2>
              <span className="text-xs text-muted">{disks.length} accessible</span>
            </div>
            <div className="grid gap-3">
              {disks.map((disk) => (
                <Card key={disk.id} className="flex items-center gap-4 p-4 shadow-none">
                  <Database aria-hidden="true" className="text-muted" size={18} />
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{disk.name || "Unnamed disk"}</p>
                    <p className="truncate font-mono text-xs text-muted">{disk.mountPath}</p>
                  </div>
                  <div className="w-52">
                    <div className="mb-1 flex justify-between text-xs text-muted">
                      <span>{formatBytes(disk.usedBytes)} used</span><span>{disk.percentageUsed.toFixed(0)}%</span>
                    </div>
                    <div className="h-1.5 overflow-hidden rounded-full bg-white/5">
                      <div className="h-full rounded-full bg-sage-400" style={{ width: `${Math.min(disk.percentageUsed, 100)}%` }} />
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          </section>
        </>
      )}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-xs text-muted">{label}</p>
      <p className="mt-1 text-lg font-semibold tabular-nums">{value}</p>
    </div>
  );
}
