import { create } from "zustand";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type { CommandError, DiskInfo } from "../ipc/types";

type DiskState = {
  disks: DiskInfo[];
  status: "idle" | "loading" | "ready" | "error";
  error: CommandError | null;
  refresh: () => Promise<void>;
};

export const useDiskStore = create<DiskState>((set) => ({
  disks: [],
  status: "idle",
  error: null,
  refresh: async () => {
    set({ status: "loading", error: null });
    try {
      set({ disks: await commands.listDisks(), status: "ready" });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
}));

