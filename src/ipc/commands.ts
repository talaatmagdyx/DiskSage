import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  AppLink,
  ApplicationUninstallPlan,
  ApplicationUninstallResult,
  ApplicationUninstallMode,
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
  InstalledApplication,
  OrphanedApplicationData,
  PermissionReport,
  ScanProfile,
  ScanProfileId,
  ScanSummary,
  StorageMapReport,
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
  openAppLink: (link: AppLink) => invokeCommand<void>("open_app_link", { request: { link } }),
  scanApplications: (includeSystemApps = false) =>
    invokeCommand<InstalledApplication[]>("scan_applications", { includeSystemApps }),
  revealApplication: (applicationId: string) =>
    invokeCommand<void>("reveal_application", { request: { applicationId } }),
  getPermissionReport: () => invokeCommand<PermissionReport>("get_permission_report"),
  openFullDiskAccessSettings: () => invokeCommand<void>("open_full_disk_access_settings"),
  scanOrphanedApplicationData: () =>
    invokeCommand<OrphanedApplicationData[]>("scan_orphaned_application_data"),
  scanStorageMap: (root?: string) =>
    invokeCommand<StorageMapReport>("scan_storage_map", { request: { root } }),
  createApplicationUninstallPlan: (applicationId: string, mode: ApplicationUninstallMode) =>
    invokeCommand<ApplicationUninstallPlan>("create_application_uninstall_plan", {
      request: { applicationId, mode },
    }),
  executeApplicationUninstallPlan: (planId: string, confirmationToken: string, selectedRelatedItemIds: string[] = [], typedConfirmation?: string) =>
    invokeCommand<ApplicationUninstallResult>("execute_application_uninstall_plan", {
      request: { planId, confirmationToken, selectedRelatedItemIds, typedConfirmation },
    }),
  exportDiagnostics: () => invokeCommand<DiagnosticsExport>("export_diagnostics"),
  listDisks: () => invokeCommand<DiskInfo[]>("list_disks"),
  getSettings: () => invokeCommand<AppSettings>("get_settings"),
  updateSettings: (settings: AppSettings) =>
    invokeCommand<AppSettings>("update_settings", { request: { settings } }),
  getScanProfiles: () => invokeCommand<ScanProfile[]>("get_scan_profiles"),
  startScan: (profile: ScanProfileId, excludedPaths: string[] = [], custom?: CustomScanOptions) =>
    invokeCommand<{ scanId: string }>("start_scan", { request: { profile, excludedPaths, custom } }),
  cancelScan: (scanId: string) => invokeCommand<void>("cancel_scan", { request: { scanId } }),
  getScanStatus: (scanId: string) =>
    invokeCommand<ScanSummary>("get_scan_status", { request: { scanId } }),
  getScanFindings: (scanId: string, offset = 0, limit = 100) =>
    invokeCommand<Finding[]>("get_scan_findings", { request: { scanId, offset, limit } }),
  revealItem: (scanId: string, findingId: string) =>
    invokeCommand<void>("reveal_item", { request: { scanId, findingId } }),
  createCleanupPlan: (scanId: string, findingIds: string[], action: CleanupAction = "moveToTrash") =>
    invokeCommand<CleanupPlan>("create_cleanup_plan", {
      request: { scanId, findingIds, action },
    }),
  executeCleanupPlan: (planId: string, confirmationToken: string, typedConfirmation?: string) =>
    invokeCommand<{ operationId: string }>("execute_cleanup_plan", {
      request: { planId, confirmationToken, typedConfirmation },
    }),
  cancelCleanup: (operationId: string) =>
    invokeCommand<void>("cancel_cleanup", { request: { operationId } }),
  getCleanupHistory: (offset = 0, limit = 50) =>
    invoke<CleanupSummary[]>("get_cleanup_history", { request: { offset, limit } }),
  clearCleanupHistory: () => invoke<void>("clear_cleanup_history"),
  startDuplicateScan: (roots: string[], minimumSizeBytes: number, byteForByteVerification: boolean) =>
    invokeCommand<{ scanId: string }>("start_duplicate_scan", {
      request: { roots, minimumSizeBytes, byteForByteVerification },
    }),
  cancelDuplicateScan: (scanId: string) =>
    invokeCommand<void>("cancel_duplicate_scan", { request: { scanId } }),
  getDuplicateScanStatus: (scanId: string) =>
    invokeCommand<DuplicateSummary>("get_duplicate_scan_status", { request: { scanId } }),
  getDuplicateGroups: (scanId: string, offset = 0, limit = 500) =>
    invokeCommand<DuplicateGroup[]>("get_duplicate_groups", { request: { scanId, offset, limit } }),
  revealDuplicate: (scanId: string, groupId: string, copyId: string) =>
    invokeCommand<void>("reveal_duplicate", { request: { scanId, groupId, copyId } }),
  createDuplicateCleanupPlan: (scanId: string, selections: DuplicateCleanupSelection[]) =>
    invokeCommand<DuplicateCleanupPlan>("create_duplicate_cleanup_plan", {
      request: { scanId, selections, action: "moveToTrash" },
    }),
  executeDuplicateCleanupPlan: (planId: string, confirmationToken: string) =>
    invokeCommand<{ operationId: string }>("execute_duplicate_cleanup_plan", {
      request: { planId, confirmationToken },
    }),
  cancelDuplicateCleanup: (operationId: string) =>
    invokeCommand<void>("cancel_duplicate_cleanup", { request: { operationId } }),
};
