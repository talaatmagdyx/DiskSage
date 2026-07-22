import { create } from "zustand";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type { CommandError, StorageMapReport } from "../ipc/types";

type StorageMapState = {
  status: "idle" | "scanning" | "ready" | "error";
  report: StorageMapReport | null;
  error: CommandError | null;
  scan: (root?: string) => Promise<void>;
  clear: () => void;
};

export const useStorageMapStore = create<StorageMapState>((set) => ({
  status: "idle",
  report: null,
  error: null,
  scan: async (root) => {
    set({ status: "scanning", error: null });
    try {
      const report = await commands.scanStorageMap(root);
      set({ report, status: "ready" });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  clear: () => set({ status: "idle", report: null, error: null }),
}));
