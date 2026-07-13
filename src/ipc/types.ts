export type DiskInfo = {
  id: string;
  name: string;
  mountPath: string;
  fileSystem: string;
  totalBytes: number;
  usedBytes: number;
  availableBytes: number;
  percentageUsed: number;
  removable: boolean;
};

export type AppSettings = {
  schemaVersion: number;
  defaultScanMode: "quick" | "developer" | "fullAnalysis" | "custom";
  followSymlinks: false;
  scanExternalDrives: boolean;
  scanHiddenFiles: boolean;
  maximumConcurrency: number;
  largeFileThresholdBytes: number;
  duplicateMinimumSizeBytes: number;
  duplicateVerificationMode: "fullHash" | "byteForByte";
  moveToTrashByDefault: true;
  permanentDeletionEnabled: boolean;
  preselectSafeItems: boolean;
  requireCleanupConfirmation: boolean;
  showExpertRecommendations: boolean;
  diagnosticLogging: boolean;
  theme: "system" | "light" | "dark";
  reducedMotion: boolean;
};

export type ErrorCode =
  | "INVALID_PATH"
  | "INVALID_SETTINGS"
  | "PATH_NOT_FOUND"
  | "PATH_PROTECTED"
  | "PERMISSION_DENIED"
  | "SCAN_ALREADY_RUNNING"
  | "SCAN_CANCELLED"
  | "FILESYSTEM_ERROR"
  | "HASH_ERROR"
  | "TRASH_FAILED"
  | "DELETE_FAILED"
  | "PLAN_EXPIRED"
  | "PLAN_VALIDATION_FAILED"
  | "COMMAND_UNAVAILABLE"
  | "DISK_INFO_FAILED"
  | "SERIALIZATION_FAILED"
  | "INTERNAL_ERROR";

export type CommandError = {
  code: ErrorCode;
  message: string;
  recoverable: boolean;
  path?: string;
  details?: string;
};
