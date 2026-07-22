import { create } from "zustand";
import { commands } from "../ipc/commands";
import { normalizeCommandError } from "../ipc/errors";
import type {
  ApplicationUninstallPlan,
  ApplicationUninstallResult,
  ApplicationUninstallMode,
  CommandError,
  InstalledApplication,
} from "../ipc/types";

type ApplicationStatus =
  | "idle"
  | "scanning"
  | "ready"
  | "planning"
  | "review"
  | "uninstalling"
  | "error";

type ApplicationState = {
  applications: InstalledApplication[];
  includeSystemApps: boolean;
  status: ApplicationStatus;
  error: CommandError | null;
  activeApplicationId: string | null;
  plan: ApplicationUninstallPlan | null;
  result: ApplicationUninstallResult | null;
  retryUninstallContext: { applicationId: string; mode: ApplicationUninstallMode } | null;
  scan: (includeSystemApps?: boolean) => Promise<void>;
  reveal: (applicationId: string) => Promise<void>;
  reviewUninstall: (applicationId: string, mode: ApplicationUninstallMode) => Promise<void>;
  dismissPlan: () => void;
  executePlan: (selectedRelatedItemIds?: string[], typedConfirmation?: string) => Promise<void>;
  retryUninstall: () => Promise<void>;
  clearResult: () => void;
};

export const useApplicationStore = create<ApplicationState>((set, get) => ({
  applications: [],
  includeSystemApps: false,
  status: "idle",
  error: null,
  activeApplicationId: null,
  plan: null,
  result: null,
  retryUninstallContext: null,
  scan: async (includeSystemApps = get().includeSystemApps) => {
    set({ status: "scanning", error: null, plan: null, activeApplicationId: null, retryUninstallContext: null, includeSystemApps });
    try {
      const applications = await commands.scanApplications(includeSystemApps);
      set({ applications, status: "ready" });
    } catch (error) {
      set({ status: "error", error: normalizeCommandError(error) });
    }
  },
  reveal: async (applicationId) => {
    try {
      await commands.revealApplication(applicationId);
    } catch (error) {
      set({ status: "ready", error: normalizeCommandError(error) });
    }
  },
  reviewUninstall: async (applicationId, mode) => {
    set({
      status: "planning",
      error: null,
      result: null,
      activeApplicationId: applicationId,
      retryUninstallContext: { applicationId, mode },
    });
    try {
      const plan = await commands.createApplicationUninstallPlan(applicationId, mode);
      set({ plan, status: "review" });
    } catch (error) {
      const normalized = normalizeCommandError(error);
      set({
        status: "ready",
        error: normalized,
        activeApplicationId: null,
        retryUninstallContext: normalized.recoverable ? { applicationId, mode } : null,
      });
    }
  },
  dismissPlan: () => set({ plan: null, status: "ready", activeApplicationId: null, retryUninstallContext: null }),
  executePlan: async (selectedRelatedItemIds = [], typedConfirmation) => {
    const plan = get().plan;
    if (!plan) return;
    set({ status: "uninstalling", error: null });
    try {
      const result = await commands.executeApplicationUninstallPlan(
        plan.id,
        plan.confirmationToken,
        plan.mode === "complete" ? plan.relatedItems.map((item) => item.id) : selectedRelatedItemIds,
        typedConfirmation,
      );
      let applications = get().applications.filter((application) => application.id !== result.applicationId);
      try {
        applications = await commands.scanApplications(get().includeSystemApps);
      } catch {
        // The backend result is authoritative; retain the locally updated list if refresh fails.
      }
      set({
        applications,
        plan: null,
        result,
        status: "ready",
        activeApplicationId: null,
        retryUninstallContext: null,
      });
    } catch (error) {
      const normalized = normalizeCommandError(error);
      set({
        status: "ready",
        error: normalized,
        plan: null,
        activeApplicationId: null,
        retryUninstallContext: normalized.recoverable ? get().retryUninstallContext : null,
      });
    }
  },
  retryUninstall: async () => {
    const context = get().retryUninstallContext;
    if (!context) return;
    await get().reviewUninstall(context.applicationId, context.mode);
  },
  clearResult: () => set({ result: null }),
}));
