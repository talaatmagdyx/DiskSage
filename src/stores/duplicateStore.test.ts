import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DuplicateGroup } from "../ipc/types";
import { useDuplicateStore } from "./duplicateStore";

vi.mock("../ipc/commands", () => ({
  commands: {
    startDuplicateScan: vi.fn(),
    cancelDuplicateScan: vi.fn(),
    getDuplicateGroups: vi.fn(),
    createDuplicateCleanupPlan: vi.fn(),
    executeDuplicateCleanupPlan: vi.fn(),
    cancelDuplicateCleanup: vi.fn(),
  },
}));

const group: DuplicateGroup = {
  id: "group-1",
  scanId: "scan-1",
  fileSize: 100,
  reclaimableBytes: 200,
  copies: [
    { id: "copy-a", path: "/tmp/a", displayPath: "/tmp/a" },
    { id: "copy-b", path: "/tmp/b", displayPath: "/tmp/b" },
    { id: "copy-c", path: "/tmp/c", displayPath: "/tmp/c" },
  ],
  recommendedKeepId: "copy-a",
  keepReason: "oldest",
  fullHash: "hash",
  byteForByteVerified: false,
};

describe("duplicateStore keep selection", () => {
  beforeEach(() => {
    useDuplicateStore.getState().reset();
    useDuplicateStore.setState({ scanId: "scan-1", status: "completed" });
    useDuplicateStore.getState().handleGroup(group);
  });

  it("automatically keeps the recommendation and selects only the other copies", () => {
    const state = useDuplicateStore.getState();
    expect(state.keepByGroup[group.id]).toBe("copy-a");
    expect([...state.selectedCopyIds]).toEqual(["copy-b", "copy-c"]);
  });

  it("changing the keep choice can never leave the keep copy selected for Trash", () => {
    useDuplicateStore.getState().setKeep(group.id, "copy-b");
    const state = useDuplicateStore.getState();
    expect(state.keepByGroup[group.id]).toBe("copy-b");
    expect(state.selectedCopyIds.has("copy-b")).toBe(false);
    expect(state.selectedCopyIds.has("copy-a")).toBe(true);
    expect(state.selectedCopyIds.has("copy-c")).toBe(true);
  });

  it("ignores attempts to toggle the current keep copy into Trash", () => {
    useDuplicateStore.getState().toggleTrash("copy-a");
    expect(useDuplicateStore.getState().selectedCopyIds.has("copy-a")).toBe(false);
  });
});
