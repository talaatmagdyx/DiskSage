import type {
  ApplicationUninstallPlan,
  ApplicationUninstallResult,
  AppSettings,
  CleanupPlan,
  CleanupProgress,
  CleanupSummary,
  DiskInfo,
  DuplicateCleanupPlan,
  DuplicateCleanupSelection,
  DuplicateGroup,
  DuplicateProgress,
  DuplicateSummary,
  Finding,
  InstalledApplication,
  OrphanedApplicationData,
  PermissionReport,
  ScanProfile,
  ScanProfileId,
  ScanProgress,
  ScanSummary,
  StorageMapReport,
} from "./types";

const parameters = new URLSearchParams(window.location.search);
const mode = parameters.get("release-preview");
const scenario = parameters.get("scenario");
const now = "2026-07-14T00:00:00.000Z";
const later = "2026-07-14T00:00:02.000Z";

type PreviewHandler<T> = (payload: T) => void;
const previewListeners = new Map<string, Set<PreviewHandler<unknown>>>();

export function listenForPreviewEvent<T>(event: string, handler: PreviewHandler<T>): () => void {
  const handlers = previewListeners.get(event) ?? new Set<PreviewHandler<unknown>>();
  handlers.add(handler as PreviewHandler<unknown>);
  previewListeners.set(event, handlers);
  return () => handlers.delete(handler as PreviewHandler<unknown>);
}

function emitPreviewEvent<T>(event: string, payload: T) {
  previewListeners.get(event)?.forEach((handler) => handler(payload));
}

function emitSoon<T>(event: string, payload: T, delay = 20) {
  window.setTimeout(() => emitPreviewEvent(event, payload), delay);
}

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
  duplicateMinimumSizeBytes: 1_048_576,
  duplicateVerificationMode: "fullHash",
  moveToTrashByDefault: true,
  permanentDeletionEnabled: false,
  preselectSafeItems: false,
  requireCleanupConfirmation: true,
  showExpertRecommendations: false,
  diagnosticLogging: false,
  theme: "dark",
  reducedMotion: false,
  projectRoots: scenario === "duplicates" ? ["/Fixture/Duplicates"] : [],
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

const trashScanId = "fixture-trash-scan";
const trashFinding: Finding = {
  id: "fixture-safe-cache",
  scanId: trashScanId,
  ruleId: "cache.fixture-v1",
  ruleVersion: 1,
  category: "applicationCache",
  displayName: "Controlled application cache",
  description: "A generated fixture that exercises review and Trash without touching real files.",
  path: "/Fixture/Cache/generated-cache.bin",
  displayPath: "~/Fixture/Cache/generated-cache.bin",
  itemType: "file",
  logicalSize: 67_108_864,
  allocatedSize: 67_108_864,
  modifiedAt: now,
  risk: "safe",
  recommendedAction: "moveToTrash",
  evidence: { kind: "knownPath" },
  cleanupAllowed: true,
};

const carefulFinding: Finding = {
  id: "fixture-careful-cache",
  scanId: trashScanId,
  ruleId: "inspection.ide.xcode-derived-data-v1",
  ruleVersion: 1,
  category: "buildArtifact",
  displayName: "Xcode DerivedData",
  description: "Regenerable Xcode build and index output. Quit Xcode first; manual selection requires Careful confirmation and the next build may be slower.",
  path: "/Fixture/Library/Developer/Xcode/DerivedData",
  displayPath: "~/Library/Developer/Xcode/DerivedData",
  itemType: "directory",
  logicalSize: 6_012_954_624,
  allocatedSize: 5_905_580_032,
  modifiedAt: now,
  risk: "careful",
  recommendedAction: "moveToTrash",
  evidence: { kind: "knownPath" },
  cleanupAllowed: true,
};

const expertFinding: Finding = {
  id: "fixture-expert-docker",
  scanId: trashScanId,
  ruleId: "inspection.docker.raw-v1",
  ruleVersion: 1,
  category: "container",
  displayName: "Docker Desktop virtual disk",
  description: "Docker Desktop virtual disk inspection only. Review Docker-owned usage; DiskSage never removes Docker.raw directly.",
  path: "/Fixture/Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw",
  displayPath: "~/Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw",
  itemType: "file",
  logicalSize: 137_438_953_472,
  allocatedSize: 19_757_629_440,
  modifiedAt: now,
  risk: "expert",
  recommendedAction: "guidedCommand",
  evidence: { kind: "knownPath" },
  cleanupAllowed: false,
  cleanupBlockReason: "Expert finding: use the guided owner command; direct filesystem cleanup is unavailable.",
  guidedAction: {
    title: "Inspect Docker usage",
    command: "docker system df -v",
    explanation: "Review Docker-owned images, containers, volumes, and build cache before using Docker's own cleanup commands.",
  },
};

const previewFindings = [expertFinding, carefulFinding, trashFinding];

function scanSummary(scanId: string, profile: ScanProfileId, phase: ScanSummary["phase"], permissionDeniedCount = 0): ScanSummary {
  return {
    scanId,
    profile,
    phase,
    startedAt: now,
    completedAt: later,
    filesScanned: phase === "cancelled" ? 384 : 2_048,
    directoriesScanned: phase === "cancelled" ? 42 : 180,
    bytesExamined: phase === "cancelled" ? 48_234_496 : 734_003_200,
    findingsCount: phase === "cancelled" ? 1 : 3,
    reclaimableBytes: phase === "cancelled" ? 8_388_608 : 67_108_864,
    skippedCount: permissionDeniedCount,
    permissionDeniedCount,
    elapsedMs: phase === "cancelled" ? 420 : 1_840,
    errors: permissionDeniedCount > 0
      ? [{ code: "PERMISSION_DENIED", message: "Two fixture folders could not be read; the rest of the scan completed.", recoverable: true, path: "~/Fixture/Restricted" }]
      : [],
  };
}

const trashSummary = scanSummary(trashScanId, "quick", "completed");
let activeScanId = "fixture-active-scan";
let activeScanProfile: ScanProfileId = "quick";

const duplicateScanId = "fixture-duplicate-scan";
const duplicateGroup: DuplicateGroup = {
  id: "fixture-duplicate-group",
  scanId: duplicateScanId,
  fileSize: 12_582_912,
  reclaimableBytes: 12_582_912,
  copies: [
    { id: "fixture-copy-keep", path: "/Fixture/Duplicates/original.bin", displayPath: "~/Fixture/Duplicates/original.bin", modifiedAt: now, owner: "Fixture user" },
    { id: "fixture-copy-trash", path: "/Fixture/Duplicates/copy.bin", displayPath: "~/Fixture/Duplicates/copy.bin", modifiedAt: later, owner: "Fixture user" },
  ],
  recommendedKeepId: "fixture-copy-keep",
  keepReason: "The original fixture path is the stable copy.",
  fullHash: "fixture-blake3-hash",
  byteForByteVerified: true,
};

const duplicateSummary: DuplicateSummary = {
  scanId: duplicateScanId,
  phase: "completed",
  roots: ["/Fixture/Duplicates"],
  minimumSizeBytes: 1_048_576,
  byteForByteVerification: true,
  startedAt: now,
  completedAt: later,
  filesScanned: 12,
  candidateFiles: 2,
  duplicateFiles: 2,
  bytesHashed: 25_165_824,
  groupsFound: 1,
  reclaimableBytes: 12_582_912,
  skippedCount: 0,
  permissionDeniedCount: 0,
  elapsedMs: 730,
  errors: [],
};

let previewApplications: InstalledApplication[] = [
  {
    id: "fixture-application-removable",
    name: "Sample Studio",
    bundleId: "com.example.samplestudio",
    version: "8.4.1",
    path: "/Applications/Sample Studio.app",
    displayPath: "/Applications/Sample Studio.app",
    logicalSize: 2_684_354_560,
    allocatedSize: 2_523_611_136,
    lastUsedAt: "2025-10-05T10:30:00.000Z",
    scope: "shared",
    uninstallAllowed: true,
  },
  {
    id: "fixture-application-user",
    name: "Sketch Pad",
    bundleId: "com.example.sketchpad",
    version: "3.2",
    path: "/Users/fixture/Applications/Sketch Pad.app",
    displayPath: "~/Applications/Sketch Pad.app",
    logicalSize: 438_304_768,
    allocatedSize: 421_527_552,
    scope: "user",
    uninstallAllowed: true,
  },
  {
    id: "fixture-application-system",
    name: "System Settings",
    bundleId: "com.apple.systempreferences",
    path: "/System/Applications/System Settings.app",
    displayPath: "/System/Applications/System Settings.app",
    logicalSize: 587_202_560,
    allocatedSize: 524_288_000,
    lastUsedAt: "2026-07-13T18:00:00.000Z",
    scope: "system",
    uninstallAllowed: false,
    uninstallBlockReason: "macOS system applications are protected and are list-only.",
  },
];
let previewApplicationPlanApplicationId: string | null = null;
let previewApplicationPlanMode: "appOnly" | "complete" | "deepCleanup" = "appOnly";

export async function bootstrapPreviewScenario() {
  if (mode !== "e2e" || scenario !== "trash") return;
  const [{ useFindingsStore }, { useScanStore }] = await Promise.all([
    import("../stores/findingsStore"),
    import("../stores/scanStore"),
  ]);
  useFindingsStore.setState({ scanId: trashScanId, findings: previewFindings, status: "ready", error: null });
  useScanStore.setState({ scanId: trashScanId, status: "completed", progress: null, summary: trashSummary, error: null });
}

export async function previewInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (command === "get_app_info") return { name: "DiskSage", version: "0.1.0", platform: "macos", architecture: "aarch64", buildProfile: "development", runtime: "Tauri 2", destructiveCommandsAvailable: true } as T;
  if (command === "open_app_link") return undefined as T;
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
  if (command === "export_diagnostics") return { path: "/Fixture/diagnostics.json" } as T;
  if (command === "scan_applications") {
    const includeSystemApps = args?.includeSystemApps === true;
    return previewApplications.filter((application) => includeSystemApps || application.scope !== "system") as T;
  }
  if (command === "reveal_application") return undefined as T;
  if (command === "open_installed_apps_settings") return undefined as T;
  if (command === "get_permission_report") {
    const report: PermissionReport = {
      checkedAt: now,
      fullDiskAccessLikely: scenario !== "application-permission",
      locations: [
        { label: "Home folder", displayPath: "~", access: "available", guidance: "Readable now." },
        { label: "Application Support", displayPath: "~/Library/Application Support", access: "available", guidance: "Readable now." },
        { label: "App Containers", displayPath: "~/Library/Containers", access: scenario === "application-permission" ? "limited" : "available", guidance: scenario === "application-permission" ? "Grant DiskSage Full Disk Access, then check again." : "Readable now. macOS can still require approval for individual protected items." },
        { label: "Shared Group Containers", displayPath: "~/Library/Group Containers", access: "available", guidance: "Readable now. macOS can still require approval for individual protected items." },
      ],
      note: "This is a read-only access check, not a macOS authorization guarantee.",
    };
    return report as T;
  }
  if (command === "open_full_disk_access_settings") return undefined as T;
  if (command === "scan_orphaned_application_data") {
    const leftovers: OrphanedApplicationData[] = [
      { id: "orphan-cache", path: "/Users/fixture/Library/Caches/com.example.retired", displayPath: "~/Library/Caches/com.example.retired", identifier: "com.example.retired", category: "Cache", logicalSize: 681_574_400, allocatedSize: 650_117_120, reason: "No currently scanned application directly matches this directory name. Ownership is uncertain, so this item is review-only and never selected automatically.", defaultSelected: false },
      { id: "orphan-container", path: "/Users/fixture/Library/Containers/com.example.oldnotes", displayPath: "~/Library/Containers/com.example.oldnotes", identifier: "com.example.oldnotes", category: "Container", logicalSize: 188_743_680, allocatedSize: 176_160_768, reason: "No currently scanned application directly matches this container identifier. Ownership is uncertain, so this item is review-only and never selected automatically.", defaultSelected: false },
    ];
    return leftovers as T;
  }
  if (command === "scan_storage_map") {
    const request = args?.request as { root?: string } | undefined;
    const root = request?.root ?? "/Users/fixture";
    const report: StorageMapReport = {
      root,
      displayRoot: request?.root ? "~/Documents" : "~",
      logicalSize: 146_028_888_064,
      allocatedSize: 131_533_373_440,
      filesScanned: 82_418,
      directoriesScanned: 9_122,
      permissionDeniedCount: scenario === "permission" ? 2 : 0,
      truncated: false,
      elapsedMs: 1842,
      note: "Allocated size reflects blocks currently used by analyzed files. It is not a cleanup recommendation, and APFS snapshots or Trash retention can delay free-space changes.",
      entries: [
        { id: "map-library", name: "Library", path: `${root}/Library`, displayPath: "~/Library", logicalSize: 82_887_352_320, allocatedSize: 76_188_680_192, filesScanned: 52_940, directoriesScanned: 6_182, permissionDeniedCount: 0, truncated: false },
        { id: "map-documents", name: "Documents", path: `${root}/Documents`, displayPath: "~/Documents", logicalSize: 38_654_705_664, allocatedSize: 34_359_738_368, filesScanned: 18_241, directoriesScanned: 1_820, permissionDeniedCount: 0, truncated: false },
        { id: "map-downloads", name: "Downloads", path: `${root}/Downloads`, displayPath: "~/Downloads", logicalSize: 24_486_830_080, allocatedSize: 20_984_954_880, filesScanned: 11_237, directoriesScanned: 1_120, permissionDeniedCount: scenario === "permission" ? 2 : 0, truncated: false },
      ],
    };
    return report as T;
  }
  if (command === "create_application_uninstall_plan") {
    const request = args?.request as { applicationId?: string; mode?: "appOnly" | "complete" | "deepCleanup" } | undefined;
    const application = previewApplications.find((item) => item.id === request?.applicationId);
    if (!application?.uninstallAllowed) throw { code: "PATH_PROTECTED", message: application?.uninstallBlockReason ?? "This application is protected.", recoverable: false };
    const identifiedItems: ApplicationUninstallPlan["relatedItems"] = [
      { id: "related-cache", path: "/Users/fixture/Library/Caches/com.example.samplestudio", displayPath: "~/Library/Caches/com.example.samplestudio", category: "Cache", logicalSize: 188_743_680, allocatedSize: 176_160_768, mayContainUserData: false, confidence: "identified", defaultSelected: true, reason: "Exact bundle-identifier cache." },
      { id: "related-support", path: "/Users/fixture/Library/Application Support/Sample Studio", displayPath: "~/Library/Application Support/Sample Studio", category: "Application Support", logicalSize: 92_274_688, allocatedSize: 88_080_384, mayContainUserData: true, confidence: "identified", defaultSelected: true, reason: "Exact app-name match in Application Support." },
      { id: "related-preferences", path: "/Users/fixture/Library/Preferences/com.example.samplestudio.plist", displayPath: "~/Library/Preferences/com.example.samplestudio.plist", category: "Preferences", logicalSize: 65_536, allocatedSize: 65_536, mayContainUserData: false, confidence: "identified", defaultSelected: true, reason: "Exact bundle-identifier preference file." },
    ];
    const expertItems: ApplicationUninstallPlan["relatedItems"] = [
      { id: "ambiguous-documents", path: "/Users/fixture/Documents/Sample Studio", displayPath: "~/Documents/Sample Studio", category: "Documents folder", logicalSize: 52_428_800, allocatedSize: 50_331_648, mayContainUserData: true, confidence: "ambiguous", defaultSelected: false, reason: "Exact app-name folder in Documents; it may contain user-created work." },
      { id: "ambiguous-group", path: "/Users/fixture/Library/Group Containers/group.com.example.studio", displayPath: "~/Library/Group Containers/group.com.example.studio", category: "Shared Group Container", logicalSize: 14_680_064, allocatedSize: 12_582_912, mayContainUserData: true, confidence: "ambiguous", defaultSelected: false, reason: "Declared shared container; other apps may use it." },
    ];
    const mode = request?.mode ?? "appOnly";
    const relatedItems = mode === "appOnly" ? [] : mode === "deepCleanup" ? [...identifiedItems, ...expertItems] : identifiedItems;
    const plan: ApplicationUninstallPlan = {
      id: "fixture-application-plan",
      createdAt: now,
      expiresAt: "2026-07-14T00:10:00.000Z",
      application,
      mode,
      relatedItems,
      totalExpectedBytes: (application.allocatedSize ?? application.logicalSize) + relatedItems.reduce((total, item) => total + (item.allocatedSize ?? item.logicalSize), 0),
      requiredConfirmationPhrase: mode === "deepCleanup" ? `DEEP CLEAN ${application.name}` : undefined,
      confirmationToken: "fixture-application-confirmation",
    };
    previewApplicationPlanApplicationId = application.id;
    previewApplicationPlanMode = plan.mode;
    return plan as T;
  }
  if (command === "execute_application_uninstall_plan") {
    const request = args?.request as { selectedRelatedItemIds?: string[] } | undefined;
    const selectedCount = request?.selectedRelatedItemIds?.length ?? 0;
    const application = previewApplications.find((item) => item.id === previewApplicationPlanApplicationId) ?? previewApplications[0];
    if (!application) throw { code: "PLAN_VALIDATION_FAILED", message: "The reviewed application is no longer available.", recoverable: true };
    const failedItems: ApplicationUninstallResult["failedItems"] = scenario === "application-permission" && selectedCount > 0
      ? [{
          displayPath: "~/Library/Containers/com.example.samplestudio",
          code: "PERMISSION_DENIED",
          message: "macOS blocked access to this app data. Quit the app and grant DiskSage Full Disk Access in System Settings > Privacy & Security, then review the uninstall again.",
        }]
      : [];
    previewApplications = previewApplications.filter((item) => item.id !== application.id);
    previewApplicationPlanApplicationId = null;
    const result: ApplicationUninstallResult = {
      applicationId: application.id,
      name: application.name,
      displayPath: application.displayPath,
      movedToTrash: true,
      expectedBytes: application.allocatedSize ?? application.logicalSize,
      mode: previewApplicationPlanMode,
      relatedItemsPlanned: selectedCount,
      relatedItemsMoved: selectedCount - failedItems.length,
      relatedItemsFailed: failedItems.length,
      failedPaths: failedItems.map((item) => item.displayPath),
      failedItems,
      remainingItems: failedItems.map((item) => ({
        id: "remaining-container",
        path: "/Users/fixture/Library/Containers/com.example.samplestudio",
        displayPath: item.displayPath,
        category: "Container",
        logicalSize: 33_554_432,
        allocatedSize: 31_457_280,
        mayContainUserData: true,
        confidence: "identified",
        defaultSelected: false,
        reason: "The post-uninstall verification found this item still on disk.",
      })),
    };
    return result as T;
  }

  if (command === "start_scan") {
    const request = args?.request as { profile?: ScanProfileId } | undefined;
    activeScanId = `fixture-${scenario ?? "scan"}-active`;
    activeScanProfile = request?.profile ?? "quick";
    const progress: ScanProgress = {
      scanId: activeScanId,
      phase: "scanning",
      currentPath: "~/Fixture/Scanning/cache-entry.bin",
      filesScanned: 384,
      directoriesScanned: 42,
      bytesExamined: 48_234_496,
      findingsCount: 1,
      reclaimableBytes: 8_388_608,
      skippedCount: 0,
      permissionDeniedCount: 0,
      elapsedMs: 380,
    };
    emitSoon("scan://progress", progress);
    if (scenario === "permission") emitSoon("scan://completed", scanSummary(activeScanId, activeScanProfile, "completed", 2), 90);
    return { scanId: activeScanId } as T;
  }
  if (command === "cancel_scan") {
    emitSoon("scan://cancelled", scanSummary(activeScanId, activeScanProfile, "cancelled"));
    return undefined as T;
  }
  if (command === "get_scan_status") return (scenario === "trash" ? trashSummary : scanSummary(activeScanId, activeScanProfile, "completed", scenario === "permission" ? 2 : 0)) as T;
  if (command === "get_scan_findings") return (scenario === "trash" ? previewFindings : []) as T;
  if (command === "reveal_item") return undefined as T;

  if (command === "create_cleanup_plan") {
    const plan: CleanupPlan = {
      id: "fixture-trash-plan",
      createdAt: now,
      expiresAt: "2026-07-14T00:10:00.000Z",
      action: "moveToTrash",
      items: [{
        scanId: trashScanId,
        findingId: trashFinding.id,
        ruleId: trashFinding.ruleId,
        ruleVersion: 1,
        path: trashFinding.path,
        canonicalPath: trashFinding.path,
        expectedType: "file",
        expectedSize: trashFinding.allocatedSize ?? trashFinding.logicalSize,
        expectedModifiedAt: trashFinding.modifiedAt,
        risk: "safe",
        validationToken: "fixture-validation-token",
      }],
      expectedReclaimableBytes: trashFinding.allocatedSize ?? trashFinding.logicalSize,
      riskSummary: { safe: 1, careful: 0, expert: 0 },
      confirmationToken: "fixture-confirmation-token",
    };
    return plan as T;
  }
  if (command === "execute_cleanup_plan") {
    const operationId = "fixture-trash-operation";
    const progress: CleanupProgress = { operationId, totalItems: 1, completedItems: 0, successCount: 0, failureCount: 0, skippedCount: 0, processedBytes: 0, currentPath: trashFinding.displayPath };
    const summary: CleanupSummary = {
      operationId,
      planId: "fixture-trash-plan",
      startedAt: now,
      completedAt: later,
      action: "moveToTrash",
      selectedCount: 1,
      successCount: 1,
      failureCount: 0,
      skippedCount: 0,
      expectedBytes: trashFinding.allocatedSize ?? trashFinding.logicalSize,
      actualFreeSpaceChangeBytes: 0,
      cancelled: false,
      items: [{ findingId: trashFinding.id, ruleId: trashFinding.ruleId, displayPath: trashFinding.displayPath, expectedBytes: trashFinding.allocatedSize ?? trashFinding.logicalSize, status: "movedToTrash" }],
      disks,
    };
    emitSoon("cleanup://started", progress);
    emitSoon("cleanup://item-completed", summary.items[0], 45);
    emitSoon("cleanup://completed", summary, 70);
    return { operationId } as T;
  }
  if (command === "cancel_cleanup") return undefined as T;

  if (command === "start_duplicate_scan") {
    const progress: DuplicateProgress = { ...duplicateSummary, phase: "fullHashing", currentPath: duplicateGroup.copies[1].displayPath };
    emitSoon("duplicates://progress", progress);
    emitSoon("duplicates://group", duplicateGroup, 45);
    emitSoon("duplicates://completed", duplicateSummary, 75);
    return { scanId: duplicateScanId } as T;
  }
  if (command === "cancel_duplicate_scan" || command === "reveal_duplicate" || command === "cancel_duplicate_cleanup") return undefined as T;
  if (command === "get_duplicate_scan_status") return duplicateSummary as T;
  if (command === "get_duplicate_groups") return [duplicateGroup] as T;
  if (command === "create_duplicate_cleanup_plan") {
    const request = args?.request as { selections?: DuplicateCleanupSelection[] } | undefined;
    const selection = request?.selections?.[0];
    if (!selection || selection.keepCopyId === selection.trashCopyIds[0]) throw { code: "PLAN_VALIDATION_FAILED", message: "The keep copy cannot be selected for Trash.", recoverable: true };
    const plan: DuplicateCleanupPlan = {
      id: "fixture-duplicate-plan",
      scanId: duplicateScanId,
      createdAt: now,
      expiresAt: "2026-07-14T00:10:00.000Z",
      action: "moveToTrash",
      items: [{
        groupId: duplicateGroup.id,
        copyId: "fixture-copy-trash",
        path: duplicateGroup.copies[1].path,
        canonicalPath: duplicateGroup.copies[1].path,
        expectedSize: duplicateGroup.fileSize,
        expectedModifiedAt: duplicateGroup.copies[1].modifiedAt,
        fullHash: duplicateGroup.fullHash,
        keepCopyId: "fixture-copy-keep",
        keepPath: duplicateGroup.copies[0].path,
        keepCanonicalPath: duplicateGroup.copies[0].path,
        keepModifiedAt: duplicateGroup.copies[0].modifiedAt,
        byteForByteVerified: true,
        validationToken: "fixture-duplicate-validation",
      }],
      expectedReclaimableBytes: duplicateGroup.fileSize,
      keptCopyCount: 1,
      confirmationToken: "fixture-duplicate-confirmation",
    };
    return plan as T;
  }
  if (command === "execute_duplicate_cleanup_plan") {
    const operationId = "fixture-duplicate-operation";
    const progress: CleanupProgress = { operationId, totalItems: 1, completedItems: 0, successCount: 0, failureCount: 0, skippedCount: 0, processedBytes: 0, currentPath: duplicateGroup.copies[1].displayPath };
    const summary: CleanupSummary = {
      operationId,
      planId: "fixture-duplicate-plan",
      startedAt: now,
      completedAt: later,
      action: "moveToTrash",
      selectedCount: 1,
      successCount: 1,
      failureCount: 0,
      skippedCount: 0,
      expectedBytes: duplicateGroup.fileSize,
      actualFreeSpaceChangeBytes: 0,
      cancelled: false,
      items: [{ findingId: "fixture-copy-trash", ruleId: "duplicates.content-v1", displayPath: duplicateGroup.copies[1].displayPath, expectedBytes: duplicateGroup.fileSize, status: "movedToTrash" }],
      disks,
    };
    emitSoon("duplicates://cleanup-progress", progress);
    emitSoon("duplicates://cleanup-completed", summary, 65);
    return { operationId } as T;
  }

  throw { code: "COMMAND_UNAVAILABLE", message: "This action is disabled in the release screenshot preview.", recoverable: true };
}
