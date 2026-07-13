import { Navigate, useLocation } from "react-router-dom";
import { LoaderCircle, RotateCcw } from "lucide-react";
import { useAppearance } from "../../hooks/useAppearance";
import { useSettingsStore } from "../../stores/settingsStore";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Toaster } from "../ui/Toaster";

export function AppBootstrap({ children }: { children: React.ReactNode }) {
  useAppearance();
  const location = useLocation();
  const { settings, status, error, load } = useSettingsStore();

  if (!settings && (status === "idle" || status === "loading")) {
    return <div className="grid min-h-screen place-items-center"><LoaderCircle className="animate-spin text-sage-300" aria-label="Starting DiskSage" /></div>;
  }
  if (!settings && error) {
    return (
      <div className="grid min-h-screen place-items-center p-8">
        <Card className="max-w-lg p-7 text-center" role="alert">
          <h1 className="text-xl font-semibold">DiskSage could not load your local settings</h1>
          <p className="mt-2 text-sm text-muted">{error.message}</p>
          <Button className="mt-5" onClick={() => void load()}><RotateCcw size={16} />Try again</Button>
        </Card>
      </div>
    );
  }
  if (settings && !settings.onboardingComplete && location.pathname !== "/welcome") return <Navigate to="/welcome" replace />;
  if (settings?.onboardingComplete && location.pathname === "/welcome") return <Navigate to="/" replace />;

  return <>{children}<Toaster /></>;
}
