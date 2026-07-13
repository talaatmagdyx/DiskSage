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
export type AppInfo = {
 name: string;
 version: string;
 platform: string;
 destructiveCommandsAvailable: boolean;
};

export type AppSettings = {
 schemaVersion: number;
 onboardingComplete: boolean;
  defaultScanMode: "quick" | "developer" | "fullAnalysis" | "custom";
  followSymlinks: false;
  scanExternalDrives: boolean;
  scanHiddenFiles: boolean;
  maximumConcurrency: number;
  largeFileThresholdBytes: number;
  veryLargeFileThresholdBytes: number;
  hugeFileThresholdBytes: number;
  oldFileThresholdDays: number;
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
  projectRoots: string[];
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
export type DiagnosticsExport = { path: string };

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

export type CustomScanOptions = {
  roots: string[];
  enabledCategories: RuleCategory[];
  minimumFileSizeBytes: number;
  maximumDepth: number;
  includeHiddenFiles: boolean;
  includeExternalDrives: boolean;
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

export type CleanupAction = "moveToTrash" | "permanentDelete";

export type CleanupPlanItem = {
  scanId: string;
  findingId: string;
  ruleId: string;
  ruleVersion: number;
  path: string;
  canonicalPath: string;
  expectedType: "file" | "directory" | "symlink";
  expectedSize: number;
  expectedModifiedAt?: string;
  risk: "safe" | "careful" | "expert";
  validationToken: string;
};

export type CleanupPlan = {
  id: string;
  createdAt: string;
  expiresAt: string;
  action: CleanupAction;
  items: CleanupPlanItem[];
  expectedReclaimableBytes: number;
  riskSummary: { safe: number; careful: number; expert: number };
  confirmationToken: string;
  requiredConfirmationPhrase?: string;
};

export type CleanupItemResult = {
  findingId: string;
  ruleId: string;
  displayPath: string;
  expectedBytes: number;
  status: "movedToTrash" | "permanentlyDeleted" | "skipped" | "failed";
  error?: CommandError;
};

export type CleanupProgress = {
  operationId: string;
  totalItems: number;
  completedItems: number;
  successCount: number;
  failureCount: number;
  skippedCount: number;
  processedBytes: number;
  currentPath?: string;
};

export type CleanupSummary = {
  operationId: string;
  planId: string;
  startedAt: string;
  completedAt: string;
  action: CleanupAction;
  selectedCount: number;
  successCount: number;
  failureCount: number;
  skippedCount: number;
  expectedBytes: number;
  actualFreeSpaceChangeBytes?: number;
  cancelled: boolean;
  items: CleanupItemResult[];
  disks: DiskInfo[];
};

export type DuplicateScanPhase =
  | "discovering"
  | "grouping"
  | "partialHashing"
  | "fullHashing"
  | "verifying"
  | "finalizing"
  | "completed"
  | "cancelled"
  | "failed";

export type DuplicateCopy = {
  id: string;
  path: string;
  displayPath: string;
  modifiedAt?: string;
  owner?: string;
};

export type DuplicateGroup = {
  id: string;
  scanId: string;
  fileSize: number;
  reclaimableBytes: number;
  copies: DuplicateCopy[];
  recommendedKeepId: string;
  keepReason: string;
  fullHash: string;
  byteForByteVerified: boolean;
};

export type DuplicateProgress = {
  scanId: string;
  phase: DuplicateScanPhase;
  currentPath?: string;
  filesScanned: number;
  candidateFiles: number;
  bytesHashed: number;
  groupsFound: number;
  reclaimableBytes: number;
  skippedCount: number;
  permissionDeniedCount: number;
  elapsedMs: number;
};

export type DuplicateSummary = Omit<DuplicateProgress, "currentPath"> & {
  roots: string[];
  minimumSizeBytes: number;
  byteForByteVerification: boolean;
  startedAt: string;
  completedAt?: string;
  duplicateFiles: number;
  errors: CommandError[];
};

export type DuplicateCleanupSelection = {
  groupId: string;
  keepCopyId: string;
  trashCopyIds: string[];
};

export type DuplicateCleanupPlanItem = {
  groupId: string;
  copyId: string;
  path: string;
  canonicalPath: string;
  expectedSize: number;
  expectedModifiedAt?: string;
  fullHash: string;
  keepCopyId: string;
  keepPath: string;
  keepCanonicalPath: string;
  keepModifiedAt?: string;
  byteForByteVerified: boolean;
  validationToken: string;
};

export type DuplicateCleanupPlan = {
  id: string;
  scanId: string;
  createdAt: string;
  expiresAt: string;
  action: CleanupAction;
  items: DuplicateCleanupPlanItem[];
  expectedReclaimableBytes: number;
  keptCopyCount: number;
  confirmationToken: string;
};
