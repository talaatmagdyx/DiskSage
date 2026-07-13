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

export type ScanProfileId = "quick" | "developer" | "fullAnalysis" | "custom";
export type ScanPhase =
  | "preparing"
  | "discoveringTargets"
  | "scanning"
  | "analyzing"
  | "hashing"
  | "finalizing"
  | "completed"
  | "cancelled"
  | "failed";

export type ScanProfile = {
  id: ScanProfileId;
  displayName: string;
  description: string;
  expectedDuration: string;
  available: boolean;
  warning?: string;
};

export type ScanProgress = {
  scanId: string;
  phase: ScanPhase;
  currentPath?: string;
  filesScanned: number;
  directoriesScanned: number;
  bytesExamined: number;
  findingsCount: number;
  reclaimableBytes: number;
  skippedCount: number;
  permissionDeniedCount: number;
  elapsedMs: number;
};

export type ScanSummary = Omit<ScanProgress, "currentPath"> & {
  profile: ScanProfileId;
  startedAt: string;
  completedAt?: string;
  errors: CommandError[];
};

export type RuleCategory =
  | "applicationCache"
  | "browserCache"
  | "packageManagerCache"
  | "buildArtifact"
  | "log"
  | "installer"
  | "duplicate"
  | "largeFile"
  | "oldFile"
  | "container"
  | "emulator";

export type Finding = {
  id: string;
  scanId: string;
  ruleId: string;
  ruleVersion: number;
  category: RuleCategory;
  displayName: string;
  description: string;
  path: string;
  displayPath: string;
  itemType: "file" | "directory" | "symlink";
  logicalSize: number;
  allocatedSize?: number;
  modifiedAt?: string;
  risk: "safe" | "careful" | "expert";
  recommendedAction: "moveToTrash" | "review" | "guidedCommand" | "noAction";
  evidence: { kind: string; [key: string]: unknown };
  cleanupAllowed: boolean;
  cleanupBlockReason?: string;
};
