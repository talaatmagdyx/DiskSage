import type { CommandError, ErrorCode } from "./types";

const friendlyMessages: Partial<Record<ErrorCode, string>> = {
  INVALID_SETTINGS: "Those settings are not valid. Review the safety limits.",
  DISK_INFO_FAILED: "Disk information could not be loaded. Try refreshing.",
  PERMISSION_DENIED: "DiskSage does not have permission to access that location.",
  PATH_PROTECTED: "That location is protected and cannot be cleaned.",
  SERIALIZATION_FAILED: "Local settings could not be read or saved.",
  COMMAND_UNAVAILABLE: "This feature is not available in the current build.",
};

export function normalizeCommandError(value: unknown): CommandError {
  if (typeof value === "object" && value !== null && "code" in value && "message" in value) {
    const error = value as CommandError;
    return { ...error, message: friendlyMessages[error.code] ?? error.message };
  }

  return {
    code: "INTERNAL_ERROR",
    message: "DiskSage encountered an unexpected local error.",
    recoverable: true,
  };
}
