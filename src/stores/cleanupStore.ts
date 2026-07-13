import { create } from "zustand";
import { confirm } from "@tauri-apps/plugin-dialog";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type {
  CleanupItemResult,
  CleanupPlan,
  CleanupProgress,
  CleanupSummary,
  CleanupAction,
  CommandError,
} from "../ipc/types";

type CleanupState = {
  status: "idle" | "planning" | "review" | "starting" | "running" | "complete" | "error";
  plan: CleanupPlan | null;
  operationId: string | null;
  progress: CleanupProgress | null;
  itemResults: CleanupItemResult[];
  summary: CleanupSummary | null;
  error: CommandError | null;
  createPlan: (scanId: string, findingIds: string[], action?: CleanupAction) => Promise<void>;
  executePlan: (typedConfirmation?: string) => Promise<void>;
  cancel: () => Promise<void>;
  dismissPlan: () => void;
  reset: () => void;
  handleProgress: (progress: CleanupProgress) => void;
  handleItem: (result: CleanupItemResult) => void;
  handleSummary: (summary: CleanupSummary) => void;
  handleFailure: (error: CommandError) => void;
};

const initialState = {
  status: "idle" as const,
  plan: null,
  operationId: null,
  progress: null,
  itemResults: [],
  summary: null,
  error: null,
};

export const useCleanupStore = create<CleanupState>((set, get) => ({
  ...initialState,
  createPlan: async (scanId, findingIds, action = "moveToTrash") => {
    set({ status: "planning", error: null, summary: null, itemResults: [] });
    try {
      const plan = await commands.createCleanupPlan(scanId, findingIds, action);
      set({ status: "review", plan });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  executePlan: async (typedConfirmation) => {
    const plan = get().plan;
    if (!plan) return;
    set({ status: "starting", error: null });
    try {
      if (plan.action === "permanentDelete") {
        const confirmed = await confirm(
          `Permanently delete ${plan.items.length} items (${plan.expectedReclaimableBytes.toLocaleString()} bytes)? This cannot be undone.`,
          { title: "Permanent deletion", kind: "warning" },
        );
        if (!confirmed) {
          set({ status: "review" });
          return;
        }
      }
      const { operationId } = await commands.executeCleanupPlan(plan.id, plan.confirmationToken, typedConfirmation);
      set({ status: "running", operationId });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  cancel: async () => {
    const operationId = get().operationId;
    if (!operationId) return;
    try {
      await commands.cancelCleanup(operationId);
    } catch (error) {
      set({ error: normalizeCommandError(error) });
    }
  },
  dismissPlan: () => set({ ...initialState }),
  reset: () => set({ ...initialState }),
  handleProgress: (progress) => {
    const operationId = get().operationId;
    if (operationId && progress.operationId !== operationId) return;
    set({ status: "running", operationId: progress.operationId, progress });
  },
  handleItem: (result) => set((state) => ({ itemResults: [...state.itemResults, result] })),
  handleSummary: (summary) => set({ status: "complete", summary, progress: null }),
  handleFailure: (error) => set({ status: "error", error: normalizeCommandError(error) }),
}));
