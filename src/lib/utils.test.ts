import { describe, expect, it } from "vitest";
import { formatBytes, presentStorageSize } from "./utils";

describe("formatBytes", () => {
  it("formats finite storage values", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1_073_741_824)).toBe("1.0 GB");
  });
});

describe("presentStorageSize", () => {
  it("uses allocated bytes for sparse virtual disks and preserves logical capacity", () => {
    const size = presentStorageSize(1024 ** 4, 119 * 1024 ** 3);

    expect(size.displayedBytes).toBe(119 * 1024 ** 3);
    expect(size.logicalBytes).toBe(1024 ** 4);
    expect(size.usesAllocatedSize).toBe(true);
    expect(size.hasDistinctLogicalSize).toBe(true);
  });

  it("falls back to logical bytes when allocated size is unavailable", () => {
    expect(presentStorageSize(1024).displayedBytes).toBe(1024);
    expect(presentStorageSize(1024).usesAllocatedSize).toBe(false);
  });
});
