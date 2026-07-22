import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ApplicationUninstallPlan, ApplicationUninstallResult, CleanupSummary, DuplicateGroup, ScanSummary } from "./types";

async function loadScenario(name: string) {
  window.history.replaceState({}, "", `/?release-preview=e2e&scenario=${name}`);
  vi.resetModules();
  return import("./releasePreview");
}

describe("controlled release-preview flows", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => {
    vi.useRealTimers();
    window.history.replaceState({}, "", "/");
  });

  it("starts and cancels a scan while preserving a partial summary", async () => {
    const preview = await loadScenario("cancel");
    const summaries: ScanSummary[] = [];
    preview.listenForPreviewEvent<ScanSummary>("scan://cancelled", (summary) => summaries.push(summary));

    const started = await preview.previewInvoke<{ scanId: string }>("start_scan", { request: { profile: "quick" } });
    await preview.previewInvoke("cancel_scan", { request: { scanId: started.scanId } });
    vi.runAllTimers();

    expect(summaries).toHaveLength(1);
    expect(summaries[0]).toMatchObject({ phase: "cancelled", filesScanned: 384, findingsCount: 1 });
  });

  it("completes around permission errors and reports their count", async () => {
    const preview = await loadScenario("permission");
    const summaries: ScanSummary[] = [];
    preview.listenForPreviewEvent<ScanSummary>("scan://completed", (summary) => summaries.push(summary));

    await preview.previewInvoke("start_scan", { request: { profile: "developer" } });
    vi.runAllTimers();

    expect(summaries[0].phase).toBe("completed");
    expect(summaries[0].permissionDeniedCount).toBe(2);
    expect(summaries[0].errors[0].code).toBe("PERMISSION_DENIED");
  });

  it("reviews and completes a controlled Trash plan", async () => {
    const preview = await loadScenario("trash");
    const summaries: CleanupSummary[] = [];
    preview.listenForPreviewEvent<CleanupSummary>("cleanup://completed", (summary) => summaries.push(summary));

    const plan = await preview.previewInvoke<{ id: string; confirmationToken: string }>("create_cleanup_plan", {
      request: { scanId: "fixture-trash-scan", findingIds: ["fixture-safe-cache"], action: "moveToTrash" },
    });
    await preview.previewInvoke("execute_cleanup_plan", { request: { planId: plan.id, confirmationToken: plan.confirmationToken } });
    vi.runAllTimers();

    expect(summaries[0]).toMatchObject({ action: "moveToTrash", selectedCount: 1, successCount: 1, failureCount: 0 });
    expect(summaries[0].items[0].status).toBe("movedToTrash");
  });

  it("reports actionable partial application-uninstall permission failures", async () => {
    const preview = await loadScenario("application-permission");
    const plan = await preview.previewInvoke<ApplicationUninstallPlan>("create_application_uninstall_plan", {
      request: { applicationId: "fixture-application-removable", mode: "complete" },
    });
    const result = await preview.previewInvoke<ApplicationUninstallResult>("execute_application_uninstall_plan", {
      request: {
        planId: plan.id,
        // Preview fixture does not validate the backend confirmation value: [REDACTED:API key param],
        selectedRelatedItemIds: plan.relatedItems.map((item) => item.id),
      },
    });

    expect(result).toMatchObject({ relatedItemsFailed: 1, relatedItemsMoved: 2 });
    expect(result.failedItems[0]).toMatchObject({
      code: "PERMISSION_DENIED",
      displayPath: "~/Library/Containers/com.example.samplestudio",
    });
    expect(result.failedItems[0].message).toContain("Full Disk Access");
  });

  it("finds duplicates, keeps one copy, and Trashes only the selected copy", async () => {
    const preview = await loadScenario("duplicates");
    const groups: DuplicateGroup[] = [];
    const summaries: CleanupSummary[] = [];
    preview.listenForPreviewEvent<DuplicateGroup>("duplicates://group", (group) => groups.push(group));
    preview.listenForPreviewEvent<CleanupSummary>("duplicates://cleanup-completed", (summary) => summaries.push(summary));

    await preview.previewInvoke("start_duplicate_scan", { request: { roots: ["/Fixture/Duplicates"], minimumSizeBytes: 1, byteForByteVerification: true } });
    vi.runAllTimers();
    const plan = await preview.previewInvoke<{ id: string; confirmationToken: string }>("create_duplicate_cleanup_plan", {
      request: { scanId: "fixture-duplicate-scan", selections: [{ groupId: groups[0].id, keepCopyId: "fixture-copy-keep", trashCopyIds: ["fixture-copy-trash"] }] },
    });
    await preview.previewInvoke("execute_duplicate_cleanup_plan", { request: { planId: plan.id, confirmationToken: plan.confirmationToken } });
    vi.runAllTimers();

    expect(groups[0].recommendedKeepId).toBe("fixture-copy-keep");
    expect(summaries[0].items).toHaveLength(1);
    expect(summaries[0].items[0]).toMatchObject({ findingId: "fixture-copy-trash", status: "movedToTrash" });
  });
});
