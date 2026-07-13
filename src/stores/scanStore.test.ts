import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands } from "../ipc/commands";
import type { ScanProgress, ScanSummary } from "../ipc/types";
import { useFindingsStore } from "./findingsStore";
import { useScanStore } from "./scanStore";

vi.mock("../ipc/commands", () => ({
  commands: {
    getScanProfiles: vi.fn(),
    startScan: vi.fn(),
    cancelScan: vi.fn(),
    getScanFindings: vi.fn(),
  },
}));

const progress: ScanProgress = {
  scanId: "scan-1",
  phase: "scanning",
  filesScanned: 10,
  directoriesScanned: 2,
  bytesExamined: 100,
  findingsCount: 1,
  reclaimableBytes: 80,
  skippedCount: 0,
  permissionDeniedCount: 0,
  elapsedMs: 20,
};

describe("scanStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useFindingsStore.setState({ scanId: null, findings: [], status: "idle", error: null });
    useScanStore.setState({ profiles: [], scanId: null, status: "idle", progress: null, summary: null, error: null });
  });

  it("starts and cancels a scan by opaque identifier", async () => {
    vi.mocked(commands.startScan).mockResolvedValue({ scanId: "scan-1" });
    vi.mocked(commands.cancelScan).mockResolvedValue();
    await useScanStore.getState().start("quick");
    expect(useScanStore.getState()).toMatchObject({ scanId: "scan-1", status: "running" });
    await useScanStore.getState().cancel();
    expect(commands.cancelScan).toHaveBeenCalledWith("scan-1");
  });

  it("preserves a terminal completed state after final progress", () => {
    useScanStore.setState({ scanId: "scan-1", status: "running" });
    useScanStore.getState().handleProgress({ ...progress, phase: "completed" });
    expect(useScanStore.getState().status).toBe("completed");
  });

  it("loads persisted findings when a scan completes", () => {
    vi.mocked(commands.getScanFindings).mockResolvedValue([]);
    const summary: ScanSummary = {
      ...progress,
      phase: "completed",
      profile: "quick",
      startedAt: "2026-01-01T00:00:00Z",
      completedAt: "2026-01-01T00:00:01Z",
      errors: [],
    };
    useScanStore.getState().handleSummary(summary);
    expect(useScanStore.getState().status).toBe("completed");
    expect(commands.getScanFindings).toHaveBeenCalledWith("scan-1", 0, 500);
  });

  it("forwards explicit roots and bounded options for Custom Scan", async () => {
    vi.mocked(commands.startScan).mockResolvedValue({ scanId: "custom-1" });
    const custom = {
      roots: ["/tmp/selected"],
      enabledCategories: ["largeFile" as const, "oldFile" as const],
      minimumFileSizeBytes: 1_048_576,
      maximumDepth: 12,
      includeHiddenFiles: false,
      includeExternalDrives: false,
    };
    await useScanStore.getState().start("custom", ["/tmp/selected/skip"], custom);
    expect(commands.startScan).toHaveBeenCalledWith(
      "custom",
      ["/tmp/selected/skip"],
      custom,
    );
  });
});
