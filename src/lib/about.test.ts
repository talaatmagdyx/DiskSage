import { describe, expect, it } from "vitest";
import { formatArchitecture, formatPlatform, formatSystemInformation } from "./about";

describe("About information", () => {
  it("uses friendly platform and architecture labels", () => {
    expect(formatPlatform("macos")).toBe("macOS");
    expect(formatPlatform("linux")).toBe("Linux");
    expect(formatArchitecture("aarch64")).toBe("Apple silicon / ARM64");
    expect(formatArchitecture("x86_64")).toBe("Intel / x86_64");
  });

  it("copies only non-sensitive product and runtime information", () => {
    const output = formatSystemInformation({
      name: "DiskSage",
      version: "0.1.0",
      platform: "macos",
      architecture: "aarch64",
      buildProfile: "release",
      runtime: "Tauri 2",
      destructiveCommandsAvailable: true,
    });

    expect(output).toContain("DiskSage 0.1.0");
    expect(output).toContain("Platform: macOS");
    expect(output).toContain("Privacy: Local by design");
    expect(output).not.toContain("/Users/");
    expect(output).not.toContain("username");
  });
});
