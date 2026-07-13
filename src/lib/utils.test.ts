import { describe, expect, it } from "vitest";
import { formatBytes } from "./utils";

describe("formatBytes", () => {
  it("formats finite storage values", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1_073_741_824)).toBe("1.0 GB");
  });
});

