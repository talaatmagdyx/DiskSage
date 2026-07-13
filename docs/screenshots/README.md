# Release screenshot procedure

Release screenshots use deterministic, path-free fixture data so personal disk paths never enter repository history.

1. Start the frontend with `npm run dev`.
2. Capture onboarding at `http://localhost:1420/welcome?release-preview=onboarding`.
3. Capture the overview at `http://localhost:1420/?release-preview=dashboard`.
4. Use an 1100 × 760 viewport and the dark theme.
5. Verify the images contain no browser chrome, personal paths, usernames, or unrelated windows.

The `release-preview` IPC fixture is enabled only by Vite's development mode. Production builds continue to call the Tauri Rust backend exclusively.
