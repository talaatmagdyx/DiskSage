import { create } from "zustand";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type { AppSettings, CommandError } from "../ipc/types";

type SettingsState = {
  settings: AppSettings | null;
  status: "idle" | "loading" | "ready" | "saving" | "error";
  error: CommandError | null;
  load: () => Promise<void>;
  save: (settings: AppSettings) => Promise<void>;
};

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: null,
  status: "idle",
  error: null,
  load: async () => {
    set({ status: "loading", error: null });
    try {
      set({ settings: await commands.getSettings(), status: "ready" });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  save: async (settings) => {
    set({ status: "saving", error: null });
    try {
      set({ settings: await commands.updateSettings(settings), status: "ready" });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
}));

