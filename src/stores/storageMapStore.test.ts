import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands } from "../ipc/commands";
import { useStorageMapStore } from "./storageMapStore";

vi.mock("../ipc/commands", () => ({
  commands: { scanStorageMap: vi.fn() },
}));

describe("storageMapStore", () => {
  beforeEach(() => {
    useStorageMapStore.setState({ status: "idle", report: null, error: null });
    vi.clearAllMocks();
  });

  it("stores a bounded storage map report", async () => {
    vi.mocked(commands.scanStorageMap).mockResolvedValue({
      root: "/Users/fixture",
      displayRoot: "~",
      entries: [],
      logicalSize: 10,
      allocatedSize: 8,
      filesScanned: 1,
      directoriesScanned: 1,
      permissionDeniedCount: 0,
      truncated: false,
      elapsedMs: 4,
      note: "Read only.",
    });

    await useStorageMapStore.getState().scan();

    expect(commands.scanStorageMap).toHaveBeenCalledWith(undefined);
    expect(useStorageMapStore.getState()).toMatchObject({
      status: "ready",
      report: { displayRoot: "~", allocatedSize: 8 },
    });
  });

  it("normalizes analysis errors", async () => {
    vi.mocked(commands.scanStorageMap).mockRejectedValue({
      code: "PATH_PROTECTED",
      message: "Outside Home.",
      recoverable: false,
    });

    await useStorageMapStore.getState().scan("/");

    expect(useStorageMapStore.getState()).toMatchObject({
      status: "error",
      error: { code: "PATH_PROTECTED" },
    });
  });
});
