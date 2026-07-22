export type UsageFilter = "all" | "30" | "90" | "180" | "unknown";

export function matchesApplicationUsage(
  lastUsedAt: string | undefined,
  filter: UsageFilter,
  now = Date.now(),
) {
  if (filter === "all") return true;
  if (filter === "unknown") return !lastUsedAt || Number.isNaN(new Date(lastUsedAt).getTime());
  if (!lastUsedAt) return false;
  const timestamp = new Date(lastUsedAt).getTime();
  if (Number.isNaN(timestamp)) return false;
  return now - timestamp >= Number(filter) * 24 * 60 * 60 * 1000;
}
