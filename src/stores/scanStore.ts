import { create } from "zustand";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type { CommandError, ScanProfile, ScanProfileId, ScanProgress, ScanSummary } from "../ipc/types";
import { useFindingsStore } from "./findingsStore";

type ScanState = {
  profiles: ScanProfile[];
  scanId: string | null;
  status: "idle" | "loadingProfiles" | "starting" | "running" | "cancelling" | "completed" | "cancelled" | "error";
  progress: ScanProgress | null;
  summary: ScanSummary | null;
  error: CommandError | null;
  loadProfiles: () => Promise<void>;
  start: (profile: ScanProfileId) => Promise<void>;
  cancel: () => Promise<void>;
  handleProgress: (progress: ScanProgress) => void;
  handleSummary: (summary: ScanSummary) => void;
};

export const useScanStore = create<ScanState>((set, get) => ({
  profiles: [],
  scanId: null,
  status: "idle",
  progress: null,
  summary: null,
  error: null,
  loadProfiles: async () => {
    set({ status: "loadingProfiles", error: null });
    try {
      set({ profiles: await commands.getScanProfiles(), status: "idle" });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  start: async (profile) => {
    useFindingsStore.getState().reset();
    set({ status: "starting", error: null, progress: null, summary: null });
    try {
      const { scanId } = await commands.startScan(profile);
      useFindingsStore.getState().reset(scanId);
      set({ scanId, status: "running" });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  cancel: async () => {
    const scanId = get().scanId;
    if (!scanId) return;
    set({ status: "cancelling", error: null });
    try {
      await commands.cancelScan(scanId);
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  handleProgress: (progress) => {
    if (get().scanId && get().scanId !== progress.scanId) return;
    const status = progress.phase === "completed" ? "completed" : progress.phase === "cancelled" ? "cancelled" : progress.phase === "failed" ? "error" : "running";
    set({ scanId: progress.scanId, progress, status });
  },
  handleSummary: (summary) => {
    if (get().scanId && get().scanId !== summary.scanId) return;
    const status = summary.phase === "completed" ? "completed" : summary.phase === "cancelled" ? "cancelled" : "error";
    set({ scanId: summary.scanId, summary, status });
    void useFindingsStore.getState().load(summary.scanId);
  },
}));
