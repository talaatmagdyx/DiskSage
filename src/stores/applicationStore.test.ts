import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands } from "../ipc/commands";
import type { InstalledApplication } from "../ipc/types";
import { useApplicationStore } from "./applicationStore";

vi.mock("../ipc/commands", () => ({
  commands: {
    scanApplications: vi.fn(),
    revealApplication: vi.fn(),
    createApplicationUninstallPlan: vi.fn(),
    executeApplicationUninstallPlan: vi.fn(),
  },
}));

const application: InstalledApplication = {
  id: "app-1",
  name: "Fixture App",
  bundleId: "com.example.fixture",
  version: "1.0",
  path: "/Applications/Fixture App.app",
  displayPath: "/Applications/Fixture App.app",
  logicalSize: 100,
  allocatedSize: 80,
  lastUsedAt: "2026-01-01T00:00:00Z",
  scope: "shared",
  uninstallAllowed: true,
};

describe("applicationStore", () => {
  beforeEach(() => {
    useApplicationStore.setState({
      applications: [],
      includeSystemApps: false,
      status: "idle",
      error: null,
      activeApplicationId: null,
      plan: null,
      result: null,
      retryUninstallContext: null,
    });
    vi.clearAllMocks();
  });

  it("loads installed applications", async () => {
    vi.mocked(commands.scanApplications).mockResolvedValue([application]);
    await useApplicationStore.getState().scan();
    expect(commands.scanApplications).toHaveBeenCalledWith(false);
    expect(useApplicationStore.getState()).toMatchObject({
      status: "ready",
      applications: [{ id: "app-1" }],
    });
  });

  it("includes protected system applications only when requested", async () => {
    const systemApplication: InstalledApplication = {
      ...application,
      id: "system-app",
      name: "System Settings",
      path: "/System/Applications/System Settings.app",
      displayPath: "/System/Applications/System Settings.app",
      scope: "system",
      uninstallAllowed: false,
      uninstallBlockReason: "macOS system applications are protected.",
    };
    vi.mocked(commands.scanApplications).mockResolvedValue([application, systemApplication]);

    await useApplicationStore.getState().scan(true);

    expect(commands.scanApplications).toHaveBeenCalledWith(true);
    expect(useApplicationStore.getState()).toMatchObject({
      includeSystemApps: true,
      applications: [{ id: "app-1" }, { id: "system-app" }],
    });
  });

  it("creates a fresh review plan after a recoverable running-app failure", async () => {
    useApplicationStore.setState({ applications: [application], status: "ready" });
    vi.mocked(commands.createApplicationUninstallPlan)
      .mockRejectedValueOnce({
        code: "APPLICATION_RUNNING",
        msg: "Quit Fixture App, then review the uninstall again.",
        recoverable: true,
      })
      .mockResolvedValueOnce({
        id: "retry-plan",
        createdAt: "2026-01-01T00:00:00Z",
        expiresAt: "2026-01-01T00:10:00Z",
        application,
        mode: "appOnly",
        relatedItems: [],
        totalExpectedBytes: 80,
        confirmationToken: "retry-confirmation",
      });

    await useApplicationStore.getState().reviewUninstall(application.id, "appOnly");
    expect(useApplicationStore.getState()).toMatchObject({
      status: "ready",
      error: { code: "APPLICATION_RUNNING" },
      retryUninstallContext: { applicationId: application.id, mode: "appOnly" },
    });

    await useApplicationStore.getState().retryUninstall();
    expect(commands.createApplicationUninstallPlan).toHaveBeenCalledTimes(2);
    expect(useApplicationStore.getState()).toMatchObject({
      status: "review",
      plan: { id: "retry-plan" },
    });
  });

  it("removes only the application returned by a confirmed Trash plan", async () => {
    const second = { ...application, id: "app-2", name: "Keep App" };
    useApplicationStore.setState({ applications: [application, second], status: "ready" });
    vi.mocked(commands.createApplicationUninstallPlan).mockResolvedValue({
      id: "plan-1",
      createdAt: "2026-01-01T00:00:00Z",
      expiresAt: "2026-01-01T00:10:00Z",
      application,
      mode: "appOnly",
      relatedItems: [],
      totalExpectedBytes: 80,
      confirmationToken: "confirmation-1",
    });
    vi.mocked(commands.executeApplicationUninstallPlan).mockResolvedValue({
      applicationId: application.id,
      name: application.name,
      displayPath: application.displayPath,
      movedToTrash: true,
      expectedBytes: 80,
      mode: "appOnly",
      relatedItemsPlanned: 0,
      relatedItemsMoved: 0,
      relatedItemsFailed: 0,
      failedPaths: [],
      failedItems: [],
      remainingItems: [],
    });
    vi.mocked(commands.scanApplications).mockResolvedValue([second]);

    await useApplicationStore.getState().reviewUninstall(application.id, "appOnly");
    expect(commands.createApplicationUninstallPlan).toHaveBeenCalledWith(application.id, "appOnly");
    expect(useApplicationStore.getState().status).toBe("review");
    await useApplicationStore.getState().executePlan();

    expect(commands.executeApplicationUninstallPlan).toHaveBeenCalledWith("plan-1", "confirmation-1", [], undefined);
    expect(useApplicationStore.getState()).toMatchObject({
      status: "ready",
      applications: [{ id: "app-2" }],
      result: { movedToTrash: true },
      plan: null,
    });
  });

  it("passes only explicitly selected expert items with the typed phrase", async () => {
    useApplicationStore.setState({ applications: [application], status: "ready" });
    vi.mocked(commands.createApplicationUninstallPlan).mockResolvedValue({
      id: "plan-deep",
      createdAt: "2026-01-01T00:00:00Z",
      expiresAt: "2026-01-01T00:10:00Z",
      application,
      mode: "deepCleanup",
      relatedItems: [
        {
          id: "identified-cache",
          path: "/Users/test/Library/Caches/com.example.fixture",
          displayPath: "~/Library/Caches/com.example.fixture",
          category: "Cache",
          logicalSize: 20,
          allocatedSize: 16,
          mayContainUserData: false,
          confidence: "identified",
          defaultSelected: true,
          reason: "Exact bundle identifier match.",
        },
        {
          id: "ambiguous-documents",
          path: "/Users/test/Documents/Fixture App",
          displayPath: "~/Documents/Fixture App",
          category: "Documents",
          logicalSize: 40,
          allocatedSize: 32,
          mayContainUserData: true,
          confidence: "ambiguous",
          defaultSelected: false,
          reason: "Exact app-name folder; ownership cannot be guaranteed.",
        },
      ],
      totalExpectedBytes: 128,
      confirmationToken: "confirmation-deep",
      requiredConfirmationPhrase: "DEEP CLEAN Fixture App",
    });
    vi.mocked(commands.executeApplicationUninstallPlan).mockResolvedValue({
      applicationId: application.id,
      name: application.name,
      displayPath: application.displayPath,
      movedToTrash: true,
      expectedBytes: 112,
      mode: "deepCleanup",
      relatedItemsPlanned: 1,
      relatedItemsMoved: 1,
      relatedItemsFailed: 0,
      failedPaths: [],
      failedItems: [],
      remainingItems: [],
    });
    vi.mocked(commands.scanApplications).mockResolvedValue([]);

    await useApplicationStore.getState().reviewUninstall(application.id, "deepCleanup");
    await useApplicationStore
      .getState()
      .executePlan(["ambiguous-documents"], "DEEP CLEAN Fixture App");

    expect(commands.executeApplicationUninstallPlan).toHaveBeenCalledWith(
      "plan-deep",
      "confirmation-deep",
      ["ambiguous-documents"],
      "DEEP CLEAN Fixture App",
    );
  });

  it("surfaces backend protection errors without changing the inventory", async () => {
    useApplicationStore.setState({ applications: [application], status: "ready" });
    vi.mocked(commands.createApplicationUninstallPlan).mockRejectedValue({
      code: "PATH_PROTECTED",
      message: "This application is protected.",
      recoverable: false,
    });

    await useApplicationStore.getState().reviewUninstall(application.id, "complete");
    expect(useApplicationStore.getState()).toMatchObject({
      status: "ready",
      applications: [{ id: "app-1" }],
      error: { code: "PATH_PROTECTED" },
    });
  });
});
