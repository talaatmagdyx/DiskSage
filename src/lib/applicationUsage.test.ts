import { describe, expect, it } from "vitest";
import { matchesApplicationUsage } from "./applicationUsage";

describe("matchesApplicationUsage", () => {
  const now = Date.UTC(2026, 6, 22);

  it("supports 30, 90, and 180 day inactivity boundaries", () => {
    expect(matchesApplicationUsage("2026-06-22T00:00:00.000Z", "30", now)).toBe(true);
    expect(matchesApplicationUsage("2026-06-23T00:00:00.000Z", "30", now)).toBe(false);
    expect(matchesApplicationUsage("2026-04-23T00:00:00.000Z", "90", now)).toBe(true);
    expect(matchesApplicationUsage("2026-01-23T00:00:00.000Z", "180", now)).toBe(true);
  });

  it("keeps unavailable usage metadata in its own explicit filter", () => {
    expect(matchesApplicationUsage(undefined, "unknown", now)).toBe(true);
    expect(matchesApplicationUsage(undefined, "30", now)).toBe(false);
    expect(matchesApplicationUsage("invalid", "unknown", now)).toBe(true);
  });
});
