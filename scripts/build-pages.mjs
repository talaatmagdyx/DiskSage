import { cp, mkdir, rm, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const output = path.join(root, "dist-pages");
const assets = path.join(output, "assets");
const screenshots = path.join(assets, "screenshots");

await rm(output, { recursive: true, force: true });
await cp(path.join(root, "site"), output, { recursive: true });
await mkdir(screenshots, { recursive: true });

for (const name of ["overview.png", "findings.png", "applications.png", "storage-map.png", "onboarding.png"]) {
  await cp(path.join(root, "docs", "screenshots", name), path.join(screenshots, name));
}

await cp(path.join(root, "public", "app-icon.png"), path.join(assets, "app-icon.png"));
await cp(path.join(root, "docs", "screenshots", "overview.png"), path.join(assets, "og.png"));
await writeFile(path.join(output, ".nojekyll"), "");

console.log(`GitHub Pages site built at ${output}`);
