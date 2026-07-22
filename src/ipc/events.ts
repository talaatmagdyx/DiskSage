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

const releasePreviewMode = import.meta.env.DEV ? new URLSearchParams(window.location.search).get("release-preview") : null;

async function listenForEvent<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  if (releasePreviewMode) {
    const { listenForPreviewEvent } = await import("./releasePreview");
    return listenForPreviewEvent(event, handler);
  }
  return listen<T>(event, ({ payload }) => handler(payload));
}

export async function listenForScanEvents(): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    listenForEvent<ScanProgress>("scan://progress", (payload) => useScanStore.getState().handleProgress(payload)),
    listenForEvent<Finding>("scan://finding", (payload) => useFindingsStore.getState().append(payload)),
    listenForEvent<ScanSummary>("scan://completed", (payload) => useScanStore.getState().handleSummary(payload)),
    listenForEvent<ScanSummary>("scan://cancelled", (payload) => useScanStore.getState().handleSummary(payload)),
    listenForEvent<ScanSummary>("scan://failed", (payload) => useScanStore.getState().handleSummary(payload)),
    listenForEvent<CleanupProgress>("cleanup://started", (payload) => useCleanupStore.getState().handleProgress(payload)),
    listenForEvent<CleanupProgress>("cleanup://progress", (payload) => useCleanupStore.getState().handleProgress(payload)),
    listenForEvent<CleanupItemResult>("cleanup://item-completed", (payload) => useCleanupStore.getState().handleItem(payload)),
    listenForEvent<CleanupSummary>("cleanup://completed", (payload) => {
      useCleanupStore.getState().handleSummary(payload);
      useFindingsStore.getState().remove(
        payload.items.filter((item) => item.status === "movedToTrash" || item.status === "permanentlyDeleted").map((item) => item.findingId),
      );
    }),
    listenForEvent<CommandError>("cleanup://failed", (payload) => useCleanupStore.getState().handleFailure(payload)),
  ]);
  return () => unlisteners.forEach((unlisten) => unlisten());
}

export async function listenForDuplicateEvents(): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    listenForEvent<DuplicateProgress>("duplicates://progress", (payload) => useDuplicateStore.getState().handleProgress(payload)),
    listenForEvent<DuplicateGroup>("duplicates://group", (payload) => useDuplicateStore.getState().handleGroup(payload)),
    listenForEvent<DuplicateSummary>("duplicates://completed", (payload) => useDuplicateStore.getState().handleSummary(payload)),
    listenForEvent<CommandError>("duplicates://failed", (payload) => useDuplicateStore.getState().handleFailure(payload)),
    listenForEvent<CleanupSummary>("duplicates://cleanup-completed", (payload) => useDuplicateStore.getState().handleCleanupSummary(payload)),
    listenForEvent<CleanupProgress>("duplicates://cleanup-progress", (payload) => useDuplicateStore.getState().handleCleanupProgress(payload)),
    listenForEvent<CommandError>("duplicates://cleanup-failed", (payload) => useDuplicateStore.getState().handleFailure(payload)),
  ]);
  return () => unlisteners.forEach((unlisten) => unlisten());
}
