import { create } from "zustand";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type { CommandError, Finding } from "../ipc/types";

type FindingsState = {
  scanId: string | null;
  findings: Finding[];
  status: "idle" | "loading" | "ready" | "error";
  error: CommandError | null;
  reset: (scanId?: string) => void;
  append: (finding: Finding) => void;
  remove: (findingIds: string[]) => void;
  load: (scanId: string) => Promise<void>;
};

export const useFindingsStore = create<FindingsState>((set, get) => ({
  scanId: null,
  findings: [],
  status: "idle",
  error: null,
  reset: (scanId) => set({ scanId: scanId ?? null, findings: [], status: "idle", error: null }),
  append: (finding) => {
    const current = get();
    if (current.scanId && current.scanId !== finding.scanId) return;
    if (current.findings.some((item) => item.id === finding.id)) return;
    set({ scanId: finding.scanId, findings: [...current.findings, finding], status: "ready" });
  },
  remove: (findingIds) => {
    const removed = new Set(findingIds);
    set((state) => ({ findings: state.findings.filter((finding) => !removed.has(finding.id)) }));
  },
  load: async (scanId) => {
    set({ scanId, status: "loading", error: null });
    try {
      const findings = await commands.getScanFindings(scanId, 0, 500);
      set({ scanId, findings, status: "ready" });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
}));
