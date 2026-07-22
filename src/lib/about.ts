import type { AppInfo } from "../ipc/types";

export function formatPlatform(platform: string) {
  if (platform === "macos") return "macOS";
  if (platform === "linux") return "Linux";
  return platform;
}

export function formatArchitecture(architecture: string) {
  if (architecture === "aarch64") return "Apple silicon / ARM64";
  if (architecture === "x86_64") return "Intel / x86_64";
  return architecture;
}

export function formatSystemInformation(info: AppInfo) {
  return [
    `${info.name} ${info.version}`,
    `Build: ${info.buildProfile === "release" ? "Release" : "Development"}`,
    `Platform: ${formatPlatform(info.platform)}`,
    `Architecture: ${info.architecture}`,
    `Runtime: ${info.runtime}`,
    "Privacy: Local by design",
  ].join("\n");
}

export async function copyText(text: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "");
  textarea.style.position = "fixed";
  textarea.style.opacity = "0";
  document.body.appendChild(textarea);
  textarea.select();
  const copied = document.execCommand("copy");
  textarea.remove();
  if (!copied) throw new Error("Clipboard access is unavailable.");
}
