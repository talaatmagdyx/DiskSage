import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  CleanupPlan,
  CleanupSummary,
  DiskInfo,
  Finding,
  ScanProfile,
  ScanProfileId,
  ScanSummary,
} from "./types";

export const commands = {
  listDisks: () => invoke<DiskInfo[]>("list_disks"),
  getSettings: () => invoke<AppSettings>("get_settings"),
  updateSettings: (settings: AppSettings) =>
    invoke<AppSettings>("update_settings", { request: { settings } }),
  getScanProfiles: () => invoke<ScanProfile[]>("get_scan_profiles"),
  startScan: (profile: ScanProfileId, excludedPaths: string[] = []) =>
    invoke<{ scanId: string }>("start_scan", { request: { profile, excludedPaths } }),
  cancelScan: (scanId: string) => invoke<void>("cancel_scan", { request: { scanId } }),
  getScanStatus: (scanId: string) =>
    invoke<ScanSummary>("get_scan_status", { request: { scanId } }),
  getScanFindings: (scanId: string, offset = 0, limit = 100) =>
    invoke<Finding[]>("get_scan_findings", { request: { scanId, offset, limit } }),
  revealItem: (scanId: string, findingId: string) =>
    invoke<void>("reveal_item", { request: { scanId, findingId } }),
  createCleanupPlan: (scanId: string, findingIds: string[]) =>
    invoke<CleanupPlan>("create_cleanup_plan", {
      request: { scanId, findingIds, action: "moveToTrash" },
    }),
  executeCleanupPlan: (planId: string, confirmationToken: string) =>
    invoke<{ operationId: string }>("execute_cleanup_plan", {
      request: { planId, confirmationToken },
    }),
  cancelCleanup: (operationId: string) =>
    invoke<void>("cancel_cleanup", { request: { operationId } }),
  getCleanupHistory: (offset = 0, limit = 50) =>
    invoke<CleanupSummary[]>("get_cleanup_history", { request: { offset, limit } }),
  clearCleanupHistory: () => invoke<void>("clear_cleanup_history"),
};
