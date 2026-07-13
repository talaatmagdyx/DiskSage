import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands } from "../ipc/commands";
import { useDiskStore } from "./diskStore";

vi.mock("../ipc/commands", () => ({
  commands: { listDisks: vi.fn() },
}));

describe("diskStore", () => {
  beforeEach(() => {
    useDiskStore.setState({ disks: [], status: "idle", error: null });
    vi.clearAllMocks();
  });

  it("loads flat mounted-disk records", async () => {
    vi.mocked(commands.listDisks).mockResolvedValue([
      {
        id: "/",
        name: "Macintosh HD",
        mountPath: "/",
        fileSystem: "apfs",
        totalBytes: 100,
        usedBytes: 40,
        availableBytes: 60,
        percentageUsed: 40,
        removable: false,
      },
    ]);
    await useDiskStore.getState().refresh();
    expect(useDiskStore.getState()).toMatchObject({ status: "ready", disks: [{ id: "/" }] });
  });

  it("moves to a structured error state", async () => {
    vi.mocked(commands.listDisks).mockRejectedValue({
      code: "PERMISSION_DENIED",
      message: "raw",
      recoverable: true,
    });
    await useDiskStore.getState().refresh();
    expect(useDiskStore.getState()).toMatchObject({
      status: "error",
      error: { code: "PERMISSION_DENIED" },
    });
  });
});

