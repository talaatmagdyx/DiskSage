import { Construction, ShieldCheck } from "lucide-react";
import { Card } from "../components/ui/Card";

export function FoundationPage({ title }: { title: string }) {
  return (
    <div className="mx-auto max-w-5xl px-8 py-8">
      <p className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-sage-300">Phase 1</p>
      <h1 className="text-3xl font-semibold tracking-tight">{title}</h1>
      <Card className="mt-8 grid min-h-96 place-items-center p-10 text-center">
        <div className="max-w-md">
          <div className="mx-auto grid size-14 place-items-center rounded-2xl bg-sage-400/10 text-sage-300">
            <Construction aria-hidden="true" />
          </div>
          <h2 className="mt-5 text-xl font-semibold">Deliberately not active yet</h2>
          <p className="mt-3 text-sm leading-6 text-muted">
            This capability starts in a later delivery phase after its bounded workers, cancellation, revalidation, and safety tests exist.
          </p>
          <p className="mt-5 inline-flex items-center gap-2 rounded-full border border-line px-3 py-1.5 text-xs text-muted">
            <ShieldCheck aria-hidden="true" size={14} /> No placeholder operation can touch your files
          </p>
        </div>
      </Card>
    </div>
  );
}

