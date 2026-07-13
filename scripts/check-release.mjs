import { access, readFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const read = (path) => readFile(new URL(path, root), "utf8");
const packageJson = JSON.parse(await read("package.json"));
const tauriConfig = JSON.parse(await read("src-tauri/tauri.conf.json"));
const cargoToml = await read("src-tauri/Cargo.toml");
const changelog = await read("CHANGELOG.md");
const expected = process.argv[2] ?? packageJson.version;
const failures = [];

if (packageJson.version !== expected) failures.push(`package.json is ${packageJson.version}, expected ${expected}`);
if (tauriConfig.version !== expected) failures.push(`tauri.conf.json is ${tauriConfig.version}, expected ${expected}`);
if (!new RegExp(`^version = "${expected.replaceAll(".", "\\.")}"$`, "m").test(cargoToml)) failures.push(`Cargo.toml does not declare ${expected}`);
if (!changelog.includes(`## [${expected}]`)) failures.push(`CHANGELOG.md has no ${expected} section`);

for (const path of [
  "LICENSE",
  "PRIVACY.md",
  "SECURITY.md",
  "INSTALL.md",
  "docs/release-checklist.md",
  "docs/screenshots/onboarding.png",
  "docs/screenshots/overview.png",
  "src-tauri/icons/icon.icns",
  "src-tauri/icons/icon.png",
]) {
  try { await access(new URL(path, root)); } catch { failures.push(`missing required release file: ${path}`); }
}

if (failures.length) {
  console.error(failures.map((failure) => `- ${failure}`).join("\n"));
  process.exitCode = 1;
} else {
  console.log(`DiskSage ${expected} release metadata is internally consistent.`);
}
