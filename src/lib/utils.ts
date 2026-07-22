import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** exponent;
  return `${value.toFixed(value >= 100 || exponent === 0 ? 0 : 1)} ${units[exponent]}`;
}

export type StorageSizePresentation = {
  displayedBytes: number;
  logicalBytes: number;
  usesAllocatedSize: boolean;
  hasDistinctLogicalSize: boolean;
};

export function presentStorageSize(
  logicalSize: number,
  allocatedSize?: number,
): StorageSizePresentation {
  const logicalBytes = Number.isFinite(logicalSize) && logicalSize > 0 ? logicalSize : 0;
  const normalizedAllocated = allocatedSize !== undefined && Number.isFinite(allocatedSize) && allocatedSize >= 0
    ? allocatedSize
    : undefined;
  const usesAllocatedSize = normalizedAllocated !== undefined;
  const displayedBytes = normalizedAllocated ?? logicalBytes;

  return {
    displayedBytes,
    logicalBytes,
    usesAllocatedSize,
    hasDistinctLogicalSize: usesAllocatedSize && logicalBytes > displayedBytes,
  };
}
