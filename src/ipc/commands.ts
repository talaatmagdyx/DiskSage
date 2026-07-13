import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, DiskInfo } from "./types";

export const commands = {
  listDisks: () => invoke<DiskInfo[]>("list_disks"),
  getSettings: () => invoke<AppSettings>("get_settings"),
  updateSettings: (settings: AppSettings) =>
    invoke<AppSettings>("update_settings", { request: { settings } }),
};

