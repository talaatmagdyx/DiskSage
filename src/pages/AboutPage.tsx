import { useEffect, useState } from "react";
import {
  BadgeCheck,
  Copy,
  ExternalLink,
  Github,
  Heart,
  LoaderCircle,
  LockKeyhole,
  RefreshCw,
  ShieldCheck,
} from "lucide-react";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type { AppInfo, AppLink } from "../ipc/types";
import { copyText, formatArchitecture, formatPlatform, formatSystemInformation } from "../lib/about";
import { toast } from "../stores/toastStore";

const resources: Array<{ link: AppLink; title: string; description: string }> = [
  { link: "privacy", title: "Privacy policy", description: "What DiskSage reads, stores, and never uploads." },
  { link: "security", title: "Security policy", description: "Security guarantees and responsible disclosure." },
  { link: "license", title: "MIT license", description: "The open-source terms for using and contributing." },
];

export function AboutPage() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      setAppInfo(await commands.getAppInfo());
    } catch (caught) {
      setError(normalizeCommandError(caught).message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const openLink = async (link: AppLink) => {
    try {
      await commands.openAppLink(link);
    } catch (caught) {
      toast({ tone: "error", title: "Link could not be opened", message: normalizeCommandError(caught).message });
    }
  };

  const copySystemInformation = async () => {
    if (!appInfo) return;
    try {
      await copyText(formatSystemInformation(appInfo));
      toast({ tone: "success", title: "System information copied", message: "Only product, platform, and build details were copied—no paths or device identifiers." });
    } catch {
      toast({ tone: "error", title: "Copy failed", message: "Clipboard access is unavailable. Try again after reopening DiskSage." });
    }
  };

  return (
    <div className="mx-auto max-w-5xl px-8 py-8">
      <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Product information</p>
      <h1 className="text-3xl font-semibold tracking-tight">About DiskSage</h1>
      <p className="mt-2 text-sm text-muted">Version, privacy commitments, support resources, and safe diagnostic details.</p>

      {loading ? (
        <Card className="mt-8 grid min-h-64 place-items-center"><LoaderCircle className="animate-spin text-sage-300" aria-label="Loading product information" /></Card>
      ) : error || !appInfo ? (
        <Card className="mt-8 border-amber-400/20 p-6" role="alert">
          <p className="font-medium">Product information could not be loaded.</p>
          <p className="mt-1 text-sm text-muted">{error}</p>
          <Button className="mt-4" variant="secondary" onClick={() => void load()}><RefreshCw size={16} />Try again</Button>
        </Card>
      ) : (
        <div className="mt-8 space-y-5">
          <Card className="overflow-hidden p-0">
            <div className="relative border-b border-line bg-[radial-gradient(circle_at_top_right,rgba(58,187,139,0.18),transparent_46%)] px-7 py-8">
              <div className="flex flex-col items-start justify-between gap-6 sm:flex-row sm:items-center">
                <div className="flex items-center gap-5">
                  <img src="/app-icon.png" className="size-20 rounded-[22px] shadow-2xl shadow-black/30" alt="DiskSage app icon" />
                  <div>
                    <div className="flex flex-wrap items-center gap-3">
                      <h2 className="text-2xl font-semibold tracking-tight">DiskSage</h2>
                      <span className="rounded-full border border-sage-400/20 bg-sage-400/10 px-2.5 py-1 text-xs font-semibold text-sage-200">v{appInfo.version}</span>
                    </div>
                    <p className="mt-2 text-sm text-muted">Understand storage. Clean up with confidence.</p>
                  </div>
                </div>
                <Button variant="secondary" onClick={() => void openLink("repository")}><Github size={16} />View source<ExternalLink size={14} /></Button>
              </div>
            </div>
            <div className="grid gap-px bg-line sm:grid-cols-3">
              <Promise icon={LockKeyhole} title="Local by design">Filesystem metadata and results stay on this device.</Promise>
              <Promise icon={ShieldCheck} title="Trash first">Reviewed items move to Trash after backend revalidation.</Promise>
              <Promise icon={BadgeCheck} title="Evidence based">Every finding keeps an explicit rule and cleanup boundary.</Promise>
            </div>
          </Card>

          <div className="grid gap-5 lg:grid-cols-[1.15fr_0.85fr]">
            <Card className="p-6">
              <div className="flex items-start justify-between gap-5">
                <div><h2 className="font-semibold">System information</h2><p className="mt-1 text-sm text-muted">Safe to include in a support request.</p></div>
                <Button variant="ghost" onClick={() => void copySystemInformation()}><Copy size={15} />Copy</Button>
              </div>
              <dl className="mt-5 divide-y divide-line rounded-xl border border-line bg-canvas/30 px-4">
                <Detail label="Version" value={appInfo.version} mono />
                <Detail label="Build" value={appInfo.buildProfile === "release" ? "Release" : "Development"} />
                <Detail label="Platform" value={formatPlatform(appInfo.platform)} />
                <Detail label="Architecture" value={formatArchitecture(appInfo.architecture)} />
                <Detail label="Runtime" value={appInfo.runtime} />
              </dl>
              <p className="mt-3 flex items-start gap-2 text-xs leading-5 text-muted"><LockKeyhole className="mt-0.5 shrink-0" size={13} aria-hidden="true" />Copying excludes usernames, paths, device identifiers, scan results, and file metadata.</p>
            </Card>

            <Card className="p-6">
              <h2 className="font-semibold">Updates</h2>
              <p className="mt-2 text-sm leading-6 text-muted">DiskSage does not download or install updates in the background. Review release notes and install a new signed build manually.</p>
              <Button className="mt-5 w-full" variant="secondary" onClick={() => void openLink("releases")}><RefreshCw size={16} />View available releases<ExternalLink size={14} /></Button>
              <div className="mt-5 rounded-xl border border-sage-400/15 bg-sage-400/[0.06] p-4">
                <p className="text-xs font-semibold text-sage-100">Current channel</p>
                <p className="mt-1 text-sm">{appInfo.buildProfile === "release" ? "Release build" : "Development build"}</p>
              </div>
            </Card>
          </div>

          <Card className="p-6">
            <h2 className="font-semibold">Policies and resources</h2>
            <p className="mt-1 text-sm text-muted">These links open in your default browser.</p>
            <div className="mt-5 grid gap-3 sm:grid-cols-3">
              {resources.map((resource) => (
                <button key={resource.link} type="button" onClick={() => void openLink(resource.link)} className="group rounded-xl border border-line bg-white/[0.025] p-4 text-left transition-colors hover:border-sage-400/30 hover:bg-sage-400/[0.05]">
                  <span className="flex items-center justify-between gap-3 font-medium">{resource.title}<ExternalLink className="text-muted transition-colors group-hover:text-sage-200" size={14} /></span>
                  <span className="mt-2 block text-xs leading-5 text-muted">{resource.description}</span>
                </button>
              ))}
            </div>
          </Card>

          <div className="flex flex-col items-center gap-2 py-4 text-center text-xs text-muted">
            <p className="flex items-center gap-1.5">Built with <Heart className="text-sage-300" size={13} aria-label="care" /> for safer disk decisions.</p>
            <p>Copyright © 2026 DiskSage contributors · MIT licensed</p>
          </div>
        </div>
      )}
    </div>
  );
}

function Promise({ icon: Icon, title, children }: { icon: typeof ShieldCheck; title: string; children: React.ReactNode }) {
  return <div className="bg-panel px-6 py-5"><Icon className="text-sage-300" size={18} aria-hidden="true" /><h3 className="mt-3 text-sm font-semibold">{title}</h3><p className="mt-1 text-xs leading-5 text-muted">{children}</p></div>;
}

function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return <div className="flex items-center justify-between gap-5 py-3 text-sm"><dt className="text-muted">{label}</dt><dd className={mono ? "font-mono" : "font-medium"}>{value}</dd></div>;
}
