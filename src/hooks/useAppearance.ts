import { useEffect } from "react";
import { useSettingsStore } from "../stores/settingsStore";

export function useAppearance() {
  const { settings, status, load } = useSettingsStore();

  useEffect(() => {
    if (status === "idle") void load();
  }, [load, status]);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      const dark = settings?.theme === "dark" || (settings?.theme !== "light" && media.matches);
      document.documentElement.classList.toggle("dark", dark);
      document.documentElement.classList.toggle("light", !dark);
      document.documentElement.dataset.reducedMotion = settings?.reducedMotion ? "true" : "false";
    };
    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [settings?.reducedMotion, settings?.theme]);
}
