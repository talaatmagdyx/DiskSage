import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Finding, ScanProgress, ScanSummary } from "./types";
import { useFindingsStore } from "../stores/findingsStore";
import { useScanStore } from "../stores/scanStore";

export async function listenForScanEvents(): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    listen<ScanProgress>("scan://progress", ({ payload }) => useScanStore.getState().handleProgress(payload)),
    listen<Finding>("scan://finding", ({ payload }) => useFindingsStore.getState().append(payload)),
    listen<ScanSummary>("scan://completed", ({ payload }) => useScanStore.getState().handleSummary(payload)),
    listen<ScanSummary>("scan://cancelled", ({ payload }) => useScanStore.getState().handleSummary(payload)),
    listen<ScanSummary>("scan://failed", ({ payload }) => useScanStore.getState().handleSummary(payload)),
  ]);
  return () => unlisteners.forEach((unlisten) => unlisten());
}

