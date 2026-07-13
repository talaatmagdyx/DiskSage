import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, DiskInfo, Finding, ScanProfile, ScanProfileId, ScanSummary } from "./types";

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
};

