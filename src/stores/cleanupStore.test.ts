import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands } from "../ipc/commands";
import type { CleanupPlan, CleanupSummary } from "../ipc/types";
import { useCleanupStore } from "./cleanupStore";
import { confirm } from "@tauri-apps/plugin-dialog";

vi.mock("../ipc/commands", () => ({
  commands: {
    createCleanupPlan: vi.fn(),
    executeCleanupPlan: vi.fn(),
    cancelCleanup: vi.fn(),
  },
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ confirm: vi.fn() }));

const plan: CleanupPlan = {
  id: "plan-1",
  createdAt: "2026-01-01T00:00:00Z",
  expiresAt: "2026-01-01T00:15:00Z",
  action: "moveToTrash",
  items: [],
  expectedReclaimableBytes: 42,
  riskSummary: { safe: 1, careful: 0, expert: 0 },
  confirmationToken: "confirmation-1",
};

describe("cleanupStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useCleanupStore.getState().reset();
  });

  it("requires a reviewed backend plan before execution", async () => {
    vi.mocked(commands.createCleanupPlan).mockResolvedValue(plan);
    vi.mocked(commands.executeCleanupPlan).mockResolvedValue({ operationId: "operation-1" });

    await useCleanupStore.getState().createPlan("scan-1", ["finding-1"]);
    expect(useCleanupStore.getState().status).toBe("review");
    expect(commands.executeCleanupPlan).not.toHaveBeenCalled();

    await useCleanupStore.getState().executePlan();
    expect(commands.executeCleanupPlan).toHaveBeenCalledWith("plan-1", "confirmation-1", undefined);
    expect(useCleanupStore.getState()).toMatchObject({ status: "running", operationId: "operation-1" });
  });

  it("keeps partial-failure counts in the completed result", () => {
    const summary: CleanupSummary = {
      operationId: "operation-1",
      planId: "plan-1",
      startedAt: "2026-01-01T00:00:00Z",
      completedAt: "2026-01-01T00:00:01Z",
      action: "moveToTrash",
      selectedCount: 3,
      successCount: 1,
      failureCount: 1,
      skippedCount: 1,
      expectedBytes: 42,
      actualFreeSpaceChangeBytes: 0,
      cancelled: false,
      items: [],
      disks: [],
    };
    useCleanupStore.getState().handleSummary(summary);
    expect(useCleanupStore.getState().summary).toMatchObject({ successCount: 1, failureCount: 1, skippedCount: 1 });
  });

  it("requires a native confirmation before permanent deletion", async () => {
    const permanentPlan: CleanupPlan = {
      ...plan,
      action: "permanentDelete",
      requiredConfirmationPhrase: "DELETE 1 EXPERT ITEMS",
      riskSummary: { safe: 0, careful: 0, expert: 1 },
    };
    vi.mocked(commands.createCleanupPlan).mockResolvedValue(permanentPlan);
    vi.mocked(commands.executeCleanupPlan).mockResolvedValue({ operationId: "operation-2" });
    vi.mocked(confirm).mockResolvedValue(true);
    await useCleanupStore.getState().createPlan("scan-1", ["finding-1"], "permanentDelete");
    await useCleanupStore.getState().executePlan("DELETE 1 EXPERT ITEMS");
    expect(confirm).toHaveBeenCalledOnce();
    expect(commands.executeCleanupPlan).toHaveBeenCalledWith(
      "plan-1",
      "confirmation-1",
      "DELETE 1 EXPERT ITEMS",
    );
  });
});
