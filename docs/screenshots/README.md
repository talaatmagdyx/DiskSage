# Release screenshot procedure

Release screenshots use deterministic, path-free fixture data so personal disk paths never enter repository history.

1. Start the frontend with `npm run dev`.
2. Use a 1280 × 720 viewport and the dark theme.
3. Capture the following routes:

   | File | Route | Interaction |
   | --- | --- | --- |
   | `onboarding.png` | `/welcome?release-preview=onboarding` | None |
   | `overview.png` | `/?release-preview=dashboard` | None |
   | `findings.png` | `/cleanup?release-preview=e2e&scenario=trash` | None |
   | `applications.png` | `/applications?release-preview=e2e` | None |
   | `storage-map.png` | `/storage-map?release-preview=e2e` | Select **Analyze Home** |

4. Verify every image is a real PNG containing no browser chrome, personal paths, usernames, notifications, or unrelated windows.
5. Confirm the README renders each image at a readable size before committing.

The `release-preview` IPC fixture is enabled only by Vite's development mode. Production builds continue to call the Tauri Rust backend exclusively.
