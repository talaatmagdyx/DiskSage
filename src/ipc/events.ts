import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  CleanupItemResult,
  CleanupProgress,
  CleanupSummary,
  CommandError,
  Finding,
  DuplicateGroup,
  DuplicateProgress,
  DuplicateSummary,
  ScanProgress,
  ScanSummary,
} from "./types";
import { useCleanupStore } from "../stores/cleanupStore";
import { useFindingsStore } from "../stores/findingsStore";
import { useScanStore } from "../stores/scanStore";
import { useDuplicateStore } from "../stores/duplicateStore";

export async function listenForScanEvents(): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    listen<ScanProgress>("scan://progress", ({ payload }) => useScanStore.getState().handleProgress(payload)),
    listen<Finding>("scan://finding", ({ payload }) => useFindingsStore.getState().append(payload)),
    listen<ScanSummary>("scan://completed", ({ payload }) => useScanStore.getState().handleSummary(payload)),
    listen<ScanSummary>("scan://cancelled", ({ payload }) => useScanStore.getState().handleSummary(payload)),
    listen<ScanSummary>("scan://failed", ({ payload }) => useScanStore.getState().handleSummary(payload)),
    listen<CleanupProgress>("cleanup://started", ({ payload }) => useCleanupStore.getState().handleProgress(payload)),
    listen<CleanupProgress>("cleanup://progress", ({ payload }) => useCleanupStore.getState().handleProgress(payload)),
    listen<CleanupItemResult>("cleanup://item-completed", ({ payload }) => useCleanupStore.getState().handleItem(payload)),
    listen<CleanupSummary>("cleanup://completed", ({ payload }) => {
      useCleanupStore.getState().handleSummary(payload);
      useFindingsStore.getState().remove(
        payload.items.filter((item) => item.status === "movedToTrash" || item.status === "permanentlyDeleted").map((item) => item.findingId),
      );
    }),
    listen<CommandError>("cleanup://failed", ({ payload }) => useCleanupStore.getState().handleFailure(payload)),
  ]);
  return () => unlisteners.forEach((unlisten) => unlisten());
}

export async function listenForDuplicateEvents(): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    listen<DuplicateProgress>("duplicates://progress", ({ payload }) => useDuplicateStore.getState().handleProgress(payload)),
    listen<DuplicateGroup>("duplicates://group", ({ payload }) => useDuplicateStore.getState().handleGroup(payload)),
    listen<DuplicateSummary>("duplicates://completed", ({ payload }) => useDuplicateStore.getState().handleSummary(payload)),
    listen<CommandError>("duplicates://failed", ({ payload }) => useDuplicateStore.getState().handleFailure(payload)),
    listen<CleanupSummary>("duplicates://cleanup-completed", ({ payload }) => useDuplicateStore.getState().handleCleanupSummary(payload)),
    listen<CleanupProgress>("duplicates://cleanup-progress", ({ payload }) => useDuplicateStore.getState().handleCleanupProgress(payload)),
    listen<CommandError>("duplicates://cleanup-failed", ({ payload }) => useDuplicateStore.getState().handleFailure(payload)),
  ]);
  return () => unlisteners.forEach((unlisten) => unlisten());
}
