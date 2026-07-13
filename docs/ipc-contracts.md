# IPC contracts

All request structs use strict deserialization with unknown fields denied. Responses use camel-case JSON. Failures use `CommandError`; frontend presentation maps its code to a user-safe message and does not render `details`.

## Registered through Phase 3

| Command | Request | Response | Side effects |
| --- | --- | --- | --- |
| `get_app_info` | none | app name, version, platform, destructive capability flag | none |
| `list_disks` | none | flat `DiskInfo[]` | reads mounted-disk metadata |
| `get_disk_info` | `{ mountPath }` | `DiskInfo` | reads mounted-disk metadata |
| `get_settings` | none | `AppSettings` | reads app-owned JSON |
| `update_settings` | `{ settings }` | validated `AppSettings` | atomically writes app-owned JSON |
| `get_scan_profiles` | none | `ScanProfile[]` | none |
| `start_scan` | `{ profile, excludedPaths }` | opaque scan ID | starts one backend-owned targeted worker |
| `cancel_scan` | `{ scanId }` | none | signals the matching cancellation token |
| `get_scan_status` | `{ scanId }` | `ScanSummary` | reads app-owned status |
| `get_scan_findings` | `{ scanId, offset, limit }` | flat `Finding[]` | pages app-owned NDJSON |
| `reveal_item` | `{ scanId, findingId }` | none | resolves the path from backend persistence and opens its parent location |
| `create_cleanup_plan` | `{ scanId, findingIds, action }` | expiring immutable `CleanupPlan` | snapshots exact backend findings; no filesystem mutation |
| `execute_cleanup_plan` | `{ planId, confirmationToken }` | opaque operation ID | consumes the plan once and starts independent Trash operations |
| `cancel_cleanup` | `{ operationId }` | none | skips remaining items after the current bounded step |
| `get_cleanup_history` | `{ offset, limit }` | `CleanupSummary[]` | pages app-owned local history |
| `clear_cleanup_history` | none | none | removes app-owned local history only |

## Deliberately unavailable

Permanent deletion and expert cleanup have no executable command path. The action enum is retained as a versioned contract, but plan creation rejects every action except `moveToTrash`.

`execute_cleanup_plan` contains only `planId` and `confirmationToken`. There is no path list or boolean `confirmed` shortcut. Scan exclusions can only reduce traversal scope; they never grant cleanup authorization.

## Event contract

Every scan or cleanup event includes its operation ID. Cleanup emits `started`, `progress`, `item-completed`, `completed`, and `failed`; per-item success is emitted only after the OS Trash operation succeeds.
