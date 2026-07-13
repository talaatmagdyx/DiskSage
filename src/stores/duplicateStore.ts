import { create } from "zustand";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type {
  CleanupSummary,
  CleanupProgress,
  CommandError,
  DuplicateCleanupPlan,
  DuplicateGroup,
  DuplicateProgress,
  DuplicateSummary,
} from "../ipc/types";

type DuplicateState = {
  scanId: string | null;
  status: "idle" | "starting" | "running" | "cancelling" | "completed" | "cancelled" | "planning" | "review" | "cleaning" | "error";
  progress: DuplicateProgress | null;
  summary: DuplicateSummary | null;
  groups: DuplicateGroup[];
  keepByGroup: Record<string, string>;
  selectedCopyIds: Set<string>;
  plan: DuplicateCleanupPlan | null;
  operationId: string | null;
  cleanupSummary: CleanupSummary | null;
  cleanupProgress: CleanupProgress | null;
  error: CommandError | null;
  start: (roots: string[], minimumSizeBytes: number, byteForByteVerification: boolean) => Promise<void>;
  cancel: () => Promise<void>;
  loadGroups: (scanId: string) => Promise<void>;
  setKeep: (groupId: string, copyId: string) => void;
  toggleTrash: (copyId: string) => void;
  createPlan: () => Promise<void>;
  dismissPlan: () => void;
  executePlan: () => Promise<void>;
  cancelCleanup: () => Promise<void>;
  handleProgress: (progress: DuplicateProgress) => void;
  handleGroup: (group: DuplicateGroup) => void;
  handleSummary: (summary: DuplicateSummary) => void;
  handleFailure: (error: CommandError) => void;
  handleCleanupSummary: (summary: CleanupSummary) => void;
  handleCleanupProgress: (progress: CleanupProgress) => void;
  reset: () => void;
};

const initial = {
  scanId: null,
  status: "idle" as const,
  progress: null,
  summary: null,
  groups: [],
  keepByGroup: {},
  selectedCopyIds: new Set<string>(),
  plan: null,
  operationId: null,
  cleanupSummary: null,
  cleanupProgress: null,
  error: null,
};

function selectionDefaults(groups: DuplicateGroup[]) {
  const keepByGroup: Record<string, string> = {};
  const selectedCopyIds = new Set<string>();
  for (const group of groups) {
    keepByGroup[group.id] = group.recommendedKeepId;
    for (const copy of group.copies) {
      if (copy.id !== group.recommendedKeepId) selectedCopyIds.add(copy.id);
    }
  }
  return { keepByGroup, selectedCopyIds };
}

export const useDuplicateStore = create<DuplicateState>((set, get) => ({
  ...initial,
  start: async (roots, minimumSizeBytes, byteForByteVerification) => {
    set({ ...initial, status: "starting" });
    try {
      const { scanId } = await commands.startDuplicateScan(roots, minimumSizeBytes, byteForByteVerification);
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
      await commands.cancelDuplicateScan(scanId);
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  loadGroups: async (scanId) => {
    try {
      const groups = await commands.getDuplicateGroups(scanId);
      set({ groups, ...selectionDefaults(groups) });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  setKeep: (groupId, copyId) => set((state) => {
    const group = state.groups.find((item) => item.id === groupId);
    if (!group?.copies.some((copy) => copy.id === copyId)) return state;
    const selectedCopyIds = new Set(state.selectedCopyIds);
    for (const copy of group.copies) {
      if (copy.id === copyId) selectedCopyIds.delete(copy.id);
      else selectedCopyIds.add(copy.id);
    }
    return { keepByGroup: { ...state.keepByGroup, [groupId]: copyId }, selectedCopyIds };
  }),
  toggleTrash: (copyId) => set((state) => {
    if (Object.values(state.keepByGroup).includes(copyId)) return state;
    const selectedCopyIds = new Set(state.selectedCopyIds);
    if (selectedCopyIds.has(copyId)) selectedCopyIds.delete(copyId); else selectedCopyIds.add(copyId);
    return { selectedCopyIds };
  }),
  createPlan: async () => {
    const state = get();
    if (!state.scanId) return;
    const selections = state.groups.flatMap((group) => {
      const keepCopyId = state.keepByGroup[group.id] ?? group.recommendedKeepId;
      const trashCopyIds = group.copies
        .filter((copy) => copy.id !== keepCopyId && state.selectedCopyIds.has(copy.id))
        .map((copy) => copy.id);
      return trashCopyIds.length ? [{ groupId: group.id, keepCopyId, trashCopyIds }] : [];
    });
    if (!selections.length) return;
    set({ status: "planning", error: null, cleanupSummary: null, cleanupProgress: null });
    try {
      const plan = await commands.createDuplicateCleanupPlan(state.scanId, selections);
      set({ status: "review", plan });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  dismissPlan: () => set({ status: "completed", plan: null }),
  executePlan: async () => {
    const plan = get().plan;
    if (!plan) return;
    set({ status: "cleaning", error: null });
    try {
      const { operationId } = await commands.executeDuplicateCleanupPlan(plan.id, plan.confirmationToken);
      set({ operationId });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  cancelCleanup: async () => {
    const operationId = get().operationId;
    if (!operationId) return;
    try {
      await commands.cancelDuplicateCleanup(operationId);
    } catch (error) {
      set({ error: normalizeCommandError(error) });
    }
  },
  handleProgress: (progress) => {
    if (get().scanId && get().scanId !== progress.scanId) return;
    const status = progress.phase === "completed" ? "completed" : progress.phase === "cancelled" ? "cancelled" : "running";
    set({ scanId: progress.scanId, progress, status });
  },
  handleGroup: (group) => set((state) => {
    if (state.scanId && state.scanId !== group.scanId) return state;
    if (state.groups.some((item) => item.id === group.id)) return state;
    const selectedCopyIds = new Set(state.selectedCopyIds);
    group.copies.forEach((copy) => {
      if (copy.id !== group.recommendedKeepId) selectedCopyIds.add(copy.id);
    });
    return {
      groups: [...state.groups, group],
      keepByGroup: { ...state.keepByGroup, [group.id]: group.recommendedKeepId },
      selectedCopyIds,
    };
  }),
  handleSummary: (summary) => {
    if (get().scanId && get().scanId !== summary.scanId) return;
    const status = summary.phase === "completed" ? "completed" : summary.phase === "cancelled" ? "cancelled" : "error";
    set({ scanId: summary.scanId, summary, status });
    if (summary.phase === "completed") void get().loadGroups(summary.scanId);
  },
  handleFailure: (error) => set({ status: "error", error: normalizeCommandError(error) }),
  handleCleanupProgress: (cleanupProgress) => {
    if (get().operationId && get().operationId !== cleanupProgress.operationId) return;
    set({ status: "cleaning", operationId: cleanupProgress.operationId, cleanupProgress });
  },
  handleCleanupSummary: (summary) => set((state) => {
    if (state.operationId && state.operationId !== summary.operationId) return state;
    const moved = new Set(summary.items.filter((item) => item.status === "movedToTrash").map((item) => item.findingId));
    const groups = state.groups
      .map((group) => ({ ...group, copies: group.copies.filter((copy) => !moved.has(copy.id)) }))
      .filter((group) => group.copies.length > 1);
    return {
      status: "completed",
      groups,
      ...selectionDefaults(groups),
      cleanupSummary: summary,
      cleanupProgress: null,
      plan: null,
      operationId: null,
    };
  }),
  reset: () => set({ ...initial, selectedCopyIds: new Set<string>() }),
}));
