import { useState } from "react";
import { ArrowLeft, ArrowRight, Check, Gauge, Laptop, LockKeyhole, Search, ShieldCheck } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import type { AppSettings } from "../ipc/types";
import { useSettingsStore } from "../stores/settingsStore";

const steps = ["Privacy", "Safety", "First scan"];

export function OnboardingPage() {
  const navigate = useNavigate();
  const { settings, status, save } = useSettingsStore();
  const [step, setStep] = useState(0);
  const [profile, setProfile] = useState<AppSettings["defaultScanMode"]>(settings?.defaultScanMode ?? "quick");

  const finish = async () => {
    if (!settings) return;
    await save({ ...settings, defaultScanMode: profile, onboardingComplete: true });
    if (useSettingsStore.getState().status === "ready") navigate("/scan", { replace: true });
  };

  return (
    <main className="grid min-h-screen place-items-center px-8 py-10">
      <div className="w-full max-w-3xl">
        <div className="mb-8 flex items-center justify-center gap-3">
          <div className="grid size-11 place-items-center rounded-xl bg-sage-400/10 text-sage-300"><ShieldCheck aria-hidden="true" /></div>
          <div><h1 className="text-xl font-bold">Welcome to DiskSage</h1><p className="text-xs text-muted">Understand storage. Clean up with confidence.</p></div>
        </div>
        <ol className="mb-5 grid grid-cols-3 gap-2" aria-label="Setup progress">
          {steps.map((label, index) => <li className={`rounded-xl border px-3 py-2 text-center text-xs ${index === step ? "border-sage-400/40 bg-sage-400/10 text-ink" : "border-line text-muted"}`} aria-current={index === step ? "step" : undefined} key={label}>{index < step ? <Check className="mr-1 inline" size={13} /> : `${index + 1}. `}{label}</li>)}
        </ol>
        <Card className="min-h-[390px] p-8">
          {step === 0 && <section aria-labelledby="privacy-title"><Laptop className="text-sage-300" size={32} /><h2 id="privacy-title" className="mt-5 text-2xl font-semibold">Your disk stays on your device.</h2><p className="mt-3 max-w-xl text-sm leading-6 text-muted">DiskSage analyzes filesystem metadata locally. It does not upload filenames, paths, hashes, scan results, or file contents.</p><div className="mt-7 grid grid-cols-2 gap-3"><Fact title="Local analysis" text="Scanning and duplicate verification run on this Mac." /><Fact title="Private diagnostics" text="Exports contain aggregate counts and redacted configuration only." /></div></section>}
          {step === 1 && <section aria-labelledby="safety-title"><LockKeyhole className="text-sage-300" size={32} /><h2 id="safety-title" className="mt-5 text-2xl font-semibold">Review comes before cleanup.</h2><p className="mt-3 max-w-xl text-sm leading-6 text-muted">A scan never removes anything. Eligible items require a separate immutable plan, a review, and an immediate safety check before they move to Trash.</p><div className="mt-7 grid grid-cols-3 gap-3"><Fact title="Trash first" text="Recoverable cleanup is the default." /><Fact title="Path revalidation" text="Targets are checked again at execution." /><Fact title="No symlink following" text="Links never widen scan or cleanup scope." /></div></section>}
          {step === 2 && <section aria-labelledby="scan-title"><Search className="text-sage-300" size={32} /><h2 id="scan-title" className="mt-5 text-2xl font-semibold">Choose your default scan.</h2><p className="mt-3 text-sm text-muted">You can use any scan type later. Quick Scan is the safest place to begin.</p><div className="mt-6 grid grid-cols-2 gap-3"><ProfileButton active={profile === "quick"} icon={Gauge} title="Quick Scan" text="Known caches and conservative targets." onClick={() => setProfile("quick")} /><ProfileButton active={profile === "developer"} icon={Search} title="Developer Scan" text="Adds configured project artifacts." onClick={() => setProfile("developer")} /></div></section>}
        </Card>
        <div className="mt-5 flex items-center justify-between">
          <Button variant="ghost" disabled={step === 0 || status === "saving"} onClick={() => setStep((value) => value - 1)}><ArrowLeft size={16} />Back</Button>
          {step < steps.length - 1 ? <Button onClick={() => setStep((value) => value + 1)}>Continue<ArrowRight size={16} /></Button> : <Button disabled={status === "saving"} onClick={() => void finish()}>{status === "saving" ? "Saving…" : "Finish setup"}<Check size={16} /></Button>}
        </div>
      </div>
    </main>
  );
}

function Fact({ title, text }: { title: string; text: string }) { return <div className="rounded-xl border border-line bg-canvas/40 p-4"><h3 className="text-sm font-semibold">{title}</h3><p className="mt-1 text-xs leading-5 text-muted">{text}</p></div>; }
function ProfileButton({ active, icon: Icon, title, text, onClick }: { active: boolean; icon: typeof Gauge; title: string; text: string; onClick: () => void }) { return <button className={`rounded-xl border p-4 text-left ${active ? "border-sage-400/50 bg-sage-400/10" : "border-line hover:bg-white/[0.03]"}`} onClick={onClick} aria-pressed={active}><Icon className="text-sage-300" size={20} /><span className="mt-3 block text-sm font-semibold">{title}</span><span className="mt-1 block text-xs text-muted">{text}</span></button>; }
