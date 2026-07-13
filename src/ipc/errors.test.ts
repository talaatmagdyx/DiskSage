import { describe, expect, it } from "vitest";
import { normalizeCommandError } from "./errors";

describe("normalizeCommandError", () => {
  it("maps structured backend failures to user-safe copy", () => {
    const result = normalizeCommandError({
      code: "DISK_INFO_FAILED",
      message: "raw platform failure",
      recoverable: true,
      details: "sensitive diagnostics",
    });
    expect(result.message).toBe("Disk information could not be loaded. Try refreshing.");
  });

  it("does not expose arbitrary thrown values", () => {
    expect(normalizeCommandError(new Error("secret path")).message).not.toContain("secret path");
  });
});

