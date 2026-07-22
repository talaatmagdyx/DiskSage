import { describe, expect, it } from "vitest";
import { normalizeCommandError } from "./errors";

describe("normalizeCommandError", () => {
  it("preserves actionable structured backend messages", () => {
    const result = normalizeCommandError({
      code: "PERMISSION_DENIED",
      message: "Grant DiskSage Full Disk Access, then try again.",
      recoverable: true,
      details: "sensitive diagnostics",
    });
    expect(result.message).toBe("Grant DiskSage Full Disk Access, then try again.");
  });

  it("accepts the Rust error field name and keeps details out of the message", () => {
    const result = normalizeCommandError({
      code: "APPLICATION_RUNNING",
      msg: "Quit Fixture App, then review the uninstall again.",
      recoverable: true,
      details: "/private/sensitive/path",
    });
    expect(result.message).toBe("Quit Fixture App, then review the uninstall again.");
    expect(result.message).not.toContain("/private/sensitive/path");
  });

  it("does not expose arbitrary thrown values", () => {
    expect(normalizeCommandError(new Error("secret path")).message).not.toContain("secret path");
  });
});
