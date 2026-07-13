import type { AppSettings, DiskInfo, ScanProfile } from "./types";

const mode = new URLSearchParams(window.location.search).get("release-preview");

let settings: AppSettings = {
  schemaVersion: 1,
  onboardingComplete: mode !== "onboarding",
  defaultScanMode: "quick",
  followSymlinks: false,
  scanExternalDrives: false,
  scanHiddenFiles: false,
  maximumConcurrency: 3,
  largeFileThresholdBytes: 1_073_741_824,
  veryLargeFileThresholdBytes: 5_368_709_120,
  hugeFileThresholdBytes: 21_474_836_480,
  oldFileThresholdDays: 365,
  duplicateMinimumSizeBytes: 10_485_760,
  duplicateVerificationMode: "fullHash",
  moveToTrashByDefault: true,
  permanentDeletionEnabled: false,
  preselectSafeItems: false,
  requireCleanupConfirmation: true,
  showExpertRecommendations: false,
  diagnosticLogging: false,
  theme: "dark",
  reducedMotion: false,
  projectRoots: [],
};

const disks: DiskInfo[] = [
  {
    id: "release-preview-disk",
    name: "Macintosh HD",
    mountPath: "/",
    fileSystem: "APFS",
    totalBytes: 1_000_204_886_016,
    usedBytes: 647_866_040_320,
    availableBytes: 352_338_845_696,
    percentageUsed: 64.8,
    removable: false,
  },
];

const profiles: ScanProfile[] = [
  { id: "quick", displayName: "Quick Scan", description: "Common low-risk caches", expectedDuration: "Usually under 30 seconds", available: true },
  { id: "developer", displayName: "Developer Scan", description: "Package caches, configured projects, IDEs, Docker, and emulators", expectedDuration: "Usually under 2 minutes", available: true },
  { id: "fullAnalysis", displayName: "Full Analysis", description: "Known rules, project artifacts, large files, and old installers", expectedDuration: "Can take significant time", available: true, warning: "Full Analysis can take significant time." },
  { id: "custom", displayName: "Custom Scan", description: "User-selected roots and rule categories", expectedDuration: "Depends on selected roots", available: true },
];

export async function previewInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (command === "get_app_info") return { name: "DiskSage", version: "0.1.0", platform: "macos", destructiveCommandsAvailable: true } as T;
  if (command === "get_settings") return settings as T;
  if (command === "update_settings") {
    const request = args?.request as { settings?: AppSettings } | undefined;
    settings = request?.settings ?? settings;
    return settings as T;
  }
  if (command === "list_disks") return disks as T;
  if (command === "get_scan_profiles") return profiles as T;
  if (command === "get_cleanup_history") return [] as T;
  if (command === "clear_cleanup_history") return undefined as T;
  throw { code: "COMMAND_UNAVAILABLE", message: "This action is disabled in the release screenshot preview.", recoverable: true };
}
