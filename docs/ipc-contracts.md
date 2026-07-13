# IPC contracts

All request structs use strict deserialization with unknown fields denied. Responses use camel-case JSON. Failures use `CommandError`; frontend presentation maps its code to a user-safe message and does not render `details`.

## Registered in Phase 1

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

## Defined but deliberately not registered

Rule, finding, scan progress, cleanup plan, create-plan request, and execute-plan request types define the cross-layer contract for later work. `CleanupPlanItem.validationToken` and `CleanupPlan.confirmationToken` are private Rust fields. They cannot be constructed by frontend deserialization.

The future `execute_cleanup_plan` request contains only `planId` and `confirmationToken`. There is no path list or boolean `confirmed` shortcut. Current scan IPC accepts exclusions, which can only reduce traversal scope; it never accepts cleanup authorization.

## Event contract

Every future event includes an operation ID. Scan progress is limited to 4–10 events per second; findings are incremental flat records, not a full filesystem tree. Event names follow the requirements' `scan://*`, `cleanup://*`, and `duplicates://*` namespaces.
