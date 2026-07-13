import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  AppSettings,
  CleanupPlan,
  CleanupSummary,
  CleanupAction,
  CustomScanOptions,
  DiskInfo,
  DuplicateCleanupPlan,
  DuplicateCleanupSelection,
  DuplicateGroup,
 DuplicateSummary,
 DiagnosticsExport,
  Finding,
  ScanProfile,
  ScanProfileId,
  ScanSummary,
} from "./types";

const releasePreviewMode = import.meta.env.DEV ? new URLSearchParams(window.location.search).get("release-preview") : null;
const invokeCommand = async <T>(command: string, args?: Record<string, unknown>) => {
  if (releasePreviewMode) {
    const { previewInvoke } = await import("./releasePreview");
    return previewInvoke<T>(command, args);
  }
  return invoke<T>(command, args);
};

export const commands = {
  getAppInfo: () => invokeCommand<AppInfo>("get_app_info"),
  exportDiagnostics: () => invokeCommand<DiagnosticsExport>("export_diagnostics"),
  listDisks: () => invokeCommand<DiskInfo[]>("list_disks"),
  getSettings: () => invokeCommand<AppSettings>("get_settings"),
  updateSettings: (settings: AppSettings) =>
    invokeCommand<AppSettings>("update_settings", { request: { settings } }),
  getScanProfiles: () => invokeCommand<ScanProfile[]>("get_scan_profiles"),
  startScan: (profile: ScanProfileId, excludedPaths: string[] = [], custom?: CustomScanOptions) =>
    invoke<{ scanId: string }>("start_scan", { request: { profile, excludedPaths, custom } }),
  cancelScan: (scanId: string) => invoke<void>("cancel_scan", { request: { scanId } }),
  getScanStatus: (scanId: string) =>
    invoke<ScanSummary>("get_scan_status", { request: { scanId } }),
  getScanFindings: (scanId: string, offset = 0, limit = 100) =>
    invoke<Finding[]>("get_scan_findings", { request: { scanId, offset, limit } }),
  revealItem: (scanId: string, findingId: string) =>
    invoke<void>("reveal_item", { request: { scanId, findingId } }),
  createCleanupPlan: (scanId: string, findingIds: string[], action: CleanupAction = "moveToTrash") =>
    invoke<CleanupPlan>("create_cleanup_plan", {
      request: { scanId, findingIds, action },
    }),
  executeCleanupPlan: (planId: string, confirmationToken: string, typedConfirmation?: string) =>
    invoke<{ operationId: string }>("execute_cleanup_plan", {
      request: { planId, confirmationToken, typedConfirmation },
    }),
  cancelCleanup: (operationId: string) =>
    invoke<void>("cancel_cleanup", { request: { operationId } }),
  getCleanupHistory: (offset = 0, limit = 50) =>
    invoke<CleanupSummary[]>("get_cleanup_history", { request: { offset, limit } }),
  clearCleanupHistory: () => invoke<void>("clear_cleanup_history"),
  startDuplicateScan: (roots: string[], minimumSizeBytes: number, byteForByteVerification: boolean) =>
    invoke<{ scanId: string }>("start_duplicate_scan", {
      request: { roots, minimumSizeBytes, byteForByteVerification },
    }),
  cancelDuplicateScan: (scanId: string) =>
    invoke<void>("cancel_duplicate_scan", { request: { scanId } }),
  getDuplicateScanStatus: (scanId: string) =>
    invoke<DuplicateSummary>("get_duplicate_scan_status", { request: { scanId } }),
  getDuplicateGroups: (scanId: string, offset = 0, limit = 500) =>
    invoke<DuplicateGroup[]>("get_duplicate_groups", { request: { scanId, offset, limit } }),
  revealDuplicate: (scanId: string, groupId: string, copyId: string) =>
    invoke<void>("reveal_duplicate", { request: { scanId, groupId, copyId } }),
  createDuplicateCleanupPlan: (scanId: string, selections: DuplicateCleanupSelection[]) =>
    invoke<DuplicateCleanupPlan>("create_duplicate_cleanup_plan", {
      request: { scanId, selections, action: "moveToTrash" },
    }),
  executeDuplicateCleanupPlan: (planId: string, confirmationToken: string) =>
    invoke<{ operationId: string }>("execute_duplicate_cleanup_plan", {
      request: { planId, confirmationToken },
    }),
  cancelDuplicateCleanup: (operationId: string) =>
    invoke<void>("cancel_duplicate_cleanup", { request: { operationId } }),
};
