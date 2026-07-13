import { useEffect } from "react";
import { CheckCircle2, Info, TriangleAlert, X, XCircle } from "lucide-react";
import { cn } from "../../lib/utils";
import { useToastStore, type ToastMessage } from "../../stores/toastStore";

const icons = {
  success: CheckCircle2,
  info: Info,
  warning: TriangleAlert,
  error: XCircle,
};

function ToastItem({ item }: { item: ToastMessage }) {
  const dismiss = useToastStore((state) => state.dismiss);
  const Icon = icons[item.tone];
  useEffect(() => {
    const timer = window.setTimeout(() => dismiss(item.id), item.tone === "error" ? 8_000 : 5_000);
    return () => window.clearTimeout(timer);
  }, [dismiss, item.id, item.tone]);

  return (
    <div
      className={cn(
        "pointer-events-auto flex w-96 max-w-[calc(100vw-2rem)] items-start gap-3 rounded-2xl border bg-panel/95 p-4 shadow-2xl backdrop-blur",
        item.tone === "error" && "border-red-400/30",
        item.tone === "warning" && "border-amber-400/30",
        item.tone === "success" && "border-sage-400/30",
      )}
      role={item.tone === "error" ? "alert" : "status"}
    >
      <Icon className={cn("mt-0.5 shrink-0", item.tone === "error" ? "text-red-300" : item.tone === "warning" ? "text-amber-300" : "text-sage-300")} aria-hidden="true" size={19} />
      <div className="min-w-0 flex-1">
        <p className="text-sm font-semibold">{item.title}</p>
        {item.message && <p className="mt-1 text-xs leading-5 text-muted">{item.message}</p>}
      </div>
      <button className="rounded-lg p-1 text-muted hover:bg-white/5 hover:text-ink" onClick={() => dismiss(item.id)} aria-label="Dismiss notification">
        <X aria-hidden="true" size={15} />
      </button>
    </div>
  );
}

export function Toaster() {
  const messages = useToastStore((state) => state.messages);
  return (
    <div className="pointer-events-none fixed bottom-5 right-5 z-[100] flex flex-col gap-3" aria-live="polite" aria-atomic="false">
      {messages.map((item) => <ToastItem key={item.id} item={item} />)}
    </div>
  );
}
