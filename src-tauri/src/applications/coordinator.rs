use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

#[cfg(target_os = "windows")]
use winreg::{
    enums::{KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY},
    RegKey, HKCU, HKLM,
};

use crate::{
    cleanup::trash_executor,
    domain::{
        application::{
            ApplicationScope, ApplicationUninstallFailure, ApplicationUninstallMode,
            ApplicationUninstallPlan, ApplicationUninstallResult,
            ExecuteApplicationUninstallRequest, InstalledApplication, RelatedApplicationItem,
            RelatedItemConfidence,
        },
        error::{CommandError, ErrorCode},
    },
};

const PLAN_LIFETIME_MINUTES: i64 = 10;
const DISKSAGE_BUNDLE_ID: &str = "com.disksage.desktop";

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileIdentity {
    canonical_path: PathBuf,
    is_directory: bool,
    modified_at: Option<std::time::SystemTime>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

#[derive(Debug, Clone)]
struct InventoryEntry {
    application: InstalledApplication,
    identity: FileIdentity,
}

#[derive(Debug, Clone)]
struct StoredPlan {
    public: ApplicationUninstallPlan,
    identity: FileIdentity,
    related_items: Vec<PlannedRelatedItem>,
    verification_items: Vec<PlannedRelatedItem>,
}

#[derive(Debug, Clone)]
struct PlannedRelatedItem {
    public: RelatedApplicationItem,
    identity: FileIdentity,
    canonical_root: PathBuf,
}

#[derive(Default)]
pub struct ApplicationManager {
    inventory: Mutex<HashMap<String, InventoryEntry>>,
    plans: Mutex<HashMap<String, StoredPlan>>,
    consumed_plans: Mutex<HashSet<String>>,
}

impl ApplicationManager {
    pub fn installed_bundle_ids(&self) -> Result<HashSet<String>, CommandError> {
        let inventory = self
            .inventory
            .lock()
            .map_err(|_| CommandError::internal("application inventory lock poisoned"))?;
        Ok(inventory
            .values()
            .filter_map(|entry| entry.application.bundle_id.clone())
            .collect())
    }

    pub fn inventory_is_empty(&self) -> Result<bool, CommandError> {
        Ok(self
            .inventory
            .lock()
            .map_err(|_| CommandError::internal("application inventory lock poisoned"))?
            .is_empty())
    }

    pub fn application(&self, application_id: &str) -> Result<InstalledApplication, CommandError> {
        self.inventory
            .lock()
            .map_err(|_| CommandError::internal("application inventory lock poisoned"))?
            .get(application_id)
            .map(|entry| entry.application.clone())
            .ok_or_else(|| {
                CommandError::new(
                    ErrorCode::PathNotFound,
                    "Scan applications again before revealing this item.",
                    true,
                )
            })
    }

    pub fn scan(
        &self,
        home: &Path,
        platform: &str,
        include_system_apps: bool,
    ) -> Result<Vec<InstalledApplication>, CommandError> {
        let entries = match platform {
            "macos" => {
                let roots = application_roots(home, include_system_apps);
                scan_roots(&roots, home)?
            }
            "windows" => scan_windows_applications(home, include_system_apps)?,
            _ => {
                return Err(CommandError::new(
                    ErrorCode::CommandUnavailable,
                    "Application inventory is currently available on macOS and Windows.",
                    true,
                ))
            }
        };
        let mut inventory = self
            .inventory
            .lock()
            .map_err(|_| CommandError::internal("application inventory lock poisoned"))?;
        inventory.clear();
        for entry in entries {
            inventory.insert(entry.application.id.clone(), entry);
        }
        let mut applications: Vec<_> = inventory
            .values()
            .map(|entry| entry.application.clone())
            .collect();
        applications.sort_by_key(|application| application.name.to_lowercase());
        Ok(applications)
    }

    pub fn create_plan(
        &self,
        application_id: &str,
        mode: ApplicationUninstallMode,
        home: &Path,
    ) -> Result<ApplicationUninstallPlan, CommandError> {
        let entry = self
            .inventory
            .lock()
            .map_err(|_| CommandError::internal("application inventory lock poisoned"))?
            .get(application_id)
            .cloned()
            .ok_or_else(|| {
                CommandError::new(
                    ErrorCode::PlanValidationFailed,
                    "Scan applications again before reviewing this uninstall.",
                    true,
                )
            })?;
        if !entry.application.uninstall_allowed {
            return Err(CommandError::new(
                ErrorCode::PathProtected,
                entry
                    .application
                    .uninstall_block_reason
                    .clone()
                    .unwrap_or_else(|| "This application is protected.".to_owned()),
                false,
            )
            .with_path(entry.application.display_path.clone()));
        }
        revalidate_application_identity(&entry.identity)?;
        ensure_application_not_running(&entry.application, &entry.identity.canonical_path)?;
        let verification_items = discover_related_items(&entry.application, home, true);
        let related_items = match mode {
            ApplicationUninstallMode::AppOnly => Vec::new(),
            ApplicationUninstallMode::Complete => verification_items
                .iter()
                .filter(|item| item.public.confidence == RelatedItemConfidence::Identified)
                .cloned()
                .collect(),
            ApplicationUninstallMode::DeepCleanup => verification_items.clone(),
        };
        let public_related_items: Vec<_> = related_items
            .iter()
            .map(|item| item.public.clone())
            .collect();
        let application_bytes = entry
            .application
            .allocated_size
            .unwrap_or(entry.application.logical_size);
        let total_expected_bytes = public_related_items
            .iter()
            .fold(application_bytes, |total, item| {
                total.saturating_add(item.allocated_size.unwrap_or(item.logical_size))
            });

        let created_at = Utc::now();
        let required_confirmation_phrase = (mode == ApplicationUninstallMode::DeepCleanup)
            .then(|| format!("DEEP CLEAN {}", entry.application.name));
        let plan = ApplicationUninstallPlan {
            id: Uuid::new_v4().to_string(),
            created_at,
            expires_at: created_at + Duration::minutes(PLAN_LIFETIME_MINUTES),
            application: entry.application,
            mode,
            related_items: public_related_items,
            total_expected_bytes,
            required_confirmation_phrase,
            confirmation_token: Uuid::new_v4().to_string(),
        };
        let stored = StoredPlan {
            public: plan.clone(),
            identity: entry.identity,
            related_items,
            verification_items,
        };
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| CommandError::internal("application plan lock poisoned"))?;
        plans.retain(|_, stored| Utc::now() < stored.public.expires_at);
        if plans.len() >= 64 {
            return Err(CommandError::new(
                ErrorCode::CommandUnavailable,
                "Too many application uninstall plans are awaiting review.",
                true,
            ));
        }
        plans.insert(plan.id.clone(), stored);
        Ok(plan)
    }

    pub fn execute(
        &self,
        request: &ExecuteApplicationUninstallRequest,
    ) -> Result<ApplicationUninstallResult, CommandError> {
        let stored = self.consume_plan(request)?;
        revalidate_application_identity(&stored.identity)?;
        ensure_application_not_running(
            &stored.public.application,
            &stored.identity.canonical_path,
        )?;
        for item in &stored.related_items {
            revalidate_related_identity(item)?;
        }
        let application = &stored.public.application;
        if !is_allowed_application_path(&stored.identity.canonical_path)
            || application.scope == ApplicationScope::System
            || application.bundle_id.as_deref() == Some(DISKSAGE_BUNDLE_ID)
        {
            return Err(CommandError::new(
                ErrorCode::PathProtected,
                "This application bundle is protected and cannot be moved by DiskSage.",
                false,
            )
            .with_path(application.display_path.clone()));
        }

        tracing::info!(application = %application.name, path = %stored.identity.canonical_path.display(), mode = ?stored.public.mode, "moving application bundle to Trash");
        if let Err(error) = trash_executor::move_to_trash(&stored.identity.canonical_path) {
            tracing::warn!(application = %application.name, path = %stored.identity.canonical_path.display(), error = %error, "application bundle could not be moved to Trash");
            return Err(error);
        }
        if stored.identity.canonical_path.exists() {
            return Err(CommandError::new(
                ErrorCode::TrashFailed,
                "macOS did not move the application bundle. Close the app and try again.",
                true,
            )
            .with_path(application.display_path.clone()));
        }
        tracing::info!(application = %application.name, "application bundle moved to Trash");
        let mut related_items_moved = 0_u64;
        let mut failed_paths = Vec::new();
        let mut failed_items = Vec::new();
        for item in &stored.related_items {
            match trash_executor::move_to_trash(&item.identity.canonical_path) {
                Ok(()) => related_items_moved += 1,
                Err(error) => {
                    tracing::warn!(path = %item.identity.canonical_path.display(), error = %error, "related application item could not be moved to Trash");
                    failed_paths.push(item.public.display_path.clone());
                    failed_items.push(ApplicationUninstallFailure {
                        display_path: item.public.display_path.clone(),
                        code: error.code,
                        message: error.message,
                    });
                }
            }
        }
        if let Ok(mut inventory) = self.inventory.lock() {
            inventory.remove(&application.id);
        }
        let remaining_items = stored
            .verification_items
            .iter()
            .filter(|item| item.identity.canonical_path.exists())
            .map(|item| item.public.clone())
            .collect();
        Ok(ApplicationUninstallResult {
            application_id: application.id.clone(),
            name: application.name.clone(),
            display_path: application.display_path.clone(),
            moved_to_trash: true,
            expected_bytes: stored.related_items.iter().fold(
                application
                    .allocated_size
                    .unwrap_or(application.logical_size),
                |total, item| {
                    total.saturating_add(
                        item.public
                            .allocated_size
                            .unwrap_or(item.public.logical_size),
                    )
                },
            ),
            mode: stored.public.mode,
            related_items_planned: stored.related_items.len() as u64,
            related_items_moved,
            related_items_failed: failed_paths.len() as u64,
            failed_paths,
            failed_items,
            remaining_items,
        })
    }

    fn consume_plan(
        &self,
        request: &ExecuteApplicationUninstallRequest,
    ) -> Result<StoredPlan, CommandError> {
        Uuid::parse_str(&request.plan_id).map_err(|_| {
            CommandError::new(
                ErrorCode::PlanValidationFailed,
                "The application uninstall plan id is invalid.",
                false,
            )
        })?;
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| CommandError::internal("application plan lock poisoned"))?;
        let mut stored = plans.get(&request.plan_id).cloned().ok_or_else(|| {
            let consumed = self
                .consumed_plans
                .lock()
                .map(|items| items.contains(&request.plan_id))
                .unwrap_or(false);
            CommandError::new(
                if consumed {
                    ErrorCode::PlanValidationFailed
                } else {
                    ErrorCode::PlanExpired
                },
                if consumed {
                    "This application uninstall plan has already been used."
                } else {
                    "This application uninstall plan is unavailable or expired."
                },
                true,
            )
        })?;
        if Utc::now() >= stored.public.expires_at {
            plans.remove(&request.plan_id);
            return Err(CommandError::new(
                ErrorCode::PlanExpired,
                "This application uninstall plan expired. Review the application again.",
                true,
            ));
        }
        if request.confirmation_token != stored.public.confirmation_token {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "The uninstall confirmation did not match the reviewed plan.",
                false,
            ));
        }
        let selected: HashSet<String> = request.selected_related_item_ids.iter().cloned().collect();
        if selected.len() != request.selected_related_item_ids.len() {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "The related-data selection contains duplicate items.",
                false,
            ));
        }
        let available: HashSet<String> = stored
            .related_items
            .iter()
            .map(|item| item.public.id.clone())
            .collect();
        if !selected.is_subset(&available) {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "The related-data selection does not match the reviewed plan.",
                false,
            ));
        }
        match stored.public.mode {
            ApplicationUninstallMode::AppOnly if !selected.is_empty() => {
                return Err(CommandError::new(
                    ErrorCode::PlanValidationFailed,
                    "App-only uninstall cannot include related data.",
                    false,
                ));
            }
            ApplicationUninstallMode::Complete if selected != available => {
                return Err(CommandError::new(
                    ErrorCode::PlanValidationFailed,
                    "The identified-data selection changed after review.",
                    false,
                ));
            }
            ApplicationUninstallMode::DeepCleanup
                if request.typed_confirmation.as_deref()
                    != stored.public.required_confirmation_phrase.as_deref() =>
            {
                return Err(CommandError::new(
                    ErrorCode::PlanValidationFailed,
                    "The expert deep-clean confirmation phrase did not match.",
                    false,
                ));
            }
            _ => {}
        }
        stored
            .related_items
            .retain(|item| selected.contains(&item.public.id));
        plans.remove(&request.plan_id);
        let mut consumed = self
            .consumed_plans
            .lock()
            .map_err(|_| CommandError::internal("consumed application plan lock poisoned"))?;
        if consumed.len() >= 512 {
            consumed.clear();
        }
        consumed.insert(request.plan_id.clone());
        Ok(stored)
    }
}

#[cfg(target_os = "windows")]
fn scan_windows_applications(
    home: &Path,
    include_system_apps: bool,
) -> Result<Vec<InventoryEntry>, CommandError> {
    const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
    let mut entries = Vec::new();
    collect_windows_registry_apps(
        HKCU,
        UNINSTALL_KEY,
        KEY_READ,
        ApplicationScope::User,
        home,
        include_system_apps,
        &mut entries,
    );
    for access in [KEY_READ | KEY_WOW64_64KEY, KEY_READ | KEY_WOW64_32KEY] {
        collect_windows_registry_apps(
            HKLM,
            UNINSTALL_KEY,
            access,
            ApplicationScope::Shared,
            home,
            include_system_apps,
            &mut entries,
        );
    }
    let mut seen = HashSet::new();
    entries.retain(|entry| {
        let application = &entry.application;
        seen.insert(format!(
            "{}\0{}\0{}",
            application.name.to_lowercase(),
            application.path.to_lowercase(),
            application.version.as_deref().unwrap_or_default()
        ))
    });
    Ok(entries)
}

#[cfg(not(target_os = "windows"))]
fn scan_windows_applications(
    _home: &Path,
    _include_system_apps: bool,
) -> Result<Vec<InventoryEntry>, CommandError> {
    Err(CommandError::new(
        ErrorCode::CommandUnavailable,
        "Windows application inventory can only run on Windows.",
        true,
    ))
}

#[cfg(target_os = "windows")]
#[allow(clippy::too_many_arguments)]
fn collect_windows_registry_apps(
    hive: &RegKey,
    key_path: &str,
    access: u32,
    default_scope: ApplicationScope,
    home: &Path,
    include_system_apps: bool,
    output: &mut Vec<InventoryEntry>,
) {
    let Ok(uninstall) = hive.open_subkey_with_flags(key_path, access) else {
        return;
    };
    for key_name in uninstall.enum_keys().flatten() {
        let Ok(key) = uninstall.open_subkey_with_flags(&key_name, access) else {
            continue;
        };
        let Ok(name) = key.get_value::<String, _>("DisplayName") else {
            continue;
        };
        if name.trim().is_empty() {
            continue;
        }
        let system_component = key.get_value::<u32, _>("SystemComponent").unwrap_or(0) != 0;
        if system_component && !include_system_apps {
            continue;
        }
        let install_location = key
            .get_value::<String, _>("InstallLocation")
            .ok()
            .and_then(|value| windows_registry_path(&value));
        let display_icon = key
            .get_value::<String, _>("DisplayIcon")
            .ok()
            .and_then(|value| windows_registry_path(&value));
        let Some(path) = install_location
            .filter(|path| path.exists())
            .or_else(|| display_icon.filter(|path| path.exists()))
        else {
            continue;
        };
        let Ok(identity) = read_only_application_identity(&path) else {
            continue;
        };
        let estimated_size =
            u64::from(key.get_value::<u32, _>("EstimatedSize").unwrap_or(0)).saturating_mul(1024);
        let version = key.get_value::<String, _>("DisplayVersion").ok();
        let scope = if system_component {
            ApplicationScope::System
        } else {
            default_scope
        };
        let id = blake3::hash(
            format!("windows-registry\0{key_name}\0{name}\0{}", path.display()).as_bytes(),
        )
        .to_hex()
        .to_string();
        output.push(InventoryEntry {
            application: InstalledApplication {
                id,
                name: name.trim().to_owned(),
                bundle_id: None,
                version,
                path: identity.canonical_path.to_string_lossy().into_owned(),
                display_path: display_path(&identity.canonical_path, home),
                logical_size: estimated_size,
                allocated_size: (estimated_size > 0).then_some(estimated_size),
                last_used_at: None,
                scope,
                uninstall_allowed: false,
                uninstall_block_reason: Some(
                    "Read-only Windows inventory. Use Settings > Apps > Installed apps to run the publisher-registered uninstaller."
                        .to_owned(),
                ),
            },
            identity,
        });
    }
}

#[cfg(target_os = "windows")]
fn windows_registry_path(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    let candidate = if let Some(remainder) = value.strip_prefix('"') {
        remainder.split('"').next().unwrap_or_default()
    } else {
        value.split(',').next().unwrap_or_default()
    }
    .trim();
    (!candidate.is_empty()).then(|| PathBuf::from(candidate))
}

#[cfg(target_os = "windows")]
fn read_only_application_identity(path: &Path) -> Result<FileIdentity, CommandError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| filesystem_error(path, error))?;
    if crate::platform::filesystem::is_link_or_reparse_point(&metadata)
        || (!metadata.is_file() && !metadata.is_dir())
    {
        return Err(CommandError::new(
            ErrorCode::InvalidPath,
            "The installed application path is redirected or unsupported.",
            true,
        )
        .with_path(path.to_string_lossy()));
    }
    let canonical_path = path
        .canonicalize()
        .map_err(|error| filesystem_error(path, error))?;
    identity_from_metadata(canonical_path, metadata)
}

fn application_roots(home: &Path, include_system_apps: bool) -> Vec<(PathBuf, ApplicationScope)> {
    let mut roots = vec![
        (home.join("Applications"), ApplicationScope::User),
        (PathBuf::from("/Applications"), ApplicationScope::Shared),
    ];
    if include_system_apps {
        roots.extend([
            (
                PathBuf::from("/System/Applications"),
                ApplicationScope::System,
            ),
            (
                PathBuf::from("/System/Library/CoreServices/Applications"),
                ApplicationScope::System,
            ),
        ]);
    }
    roots
}

fn ensure_application_not_running(
    application: &InstalledApplication,
    canonical_path: &Path,
) -> Result<(), CommandError> {
    if application_is_running(canonical_path) {
        return Err(CommandError::new(
            ErrorCode::ApplicationRunning,
            format!(
                "{} is currently running. Quit it completely, then review the uninstall again.",
                application.name
            ),
            true,
        )
        .with_path(application.display_path.clone()));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn application_is_running(canonical_path: &Path) -> bool {
    let Ok(output) = Command::new("ps").args(["-axo", "command="]).output() else {
        return false;
    };
    output.status.success()
        && process_list_contains_bundle(&String::from_utf8_lossy(&output.stdout), canonical_path)
}

#[cfg(not(target_os = "macos"))]
fn application_is_running(_canonical_path: &Path) -> bool {
    false
}

#[cfg(any(target_os = "macos", test))]
fn process_list_contains_bundle(process_list: &str, bundle_path: &Path) -> bool {
    let executable_prefix = format!("{}/Contents/MacOS/", bundle_path.to_string_lossy());
    process_list
        .lines()
        .any(|command| command.trim_start().starts_with(&executable_prefix))
}

fn scan_roots(
    roots: &[(PathBuf, ApplicationScope)],
    home: &Path,
) -> Result<Vec<InventoryEntry>, CommandError> {
    let mut paths = Vec::new();
    for (root, scope) in roots {
        discover_application_bundles(root, *scope, 0, &mut paths);
    }
    let mut entries = Vec::with_capacity(paths.len());
    for (path, scope) in paths {
        match inspect_application(&path, scope, home) {
            Ok(entry) => entries.push(entry),
            Err(error) => {
                tracing::warn!(path = %path.display(), error = %error, "application inspection skipped")
            }
        }
    }
    Ok(entries)
}

fn discover_application_bundles(
    directory: &Path,
    scope: ApplicationScope,
    depth: usize,
    output: &mut Vec<(PathBuf, ApplicationScope)>,
) {
    if depth > 2 {
        return;
    }
    let Ok(read_dir) = fs::read_dir(directory) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() || !file_type.is_dir() {
            continue;
        }
        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
        {
            output.push((path, scope));
        } else {
            discover_application_bundles(&path, scope, depth + 1, output);
        }
    }
}

fn inspect_application(
    path: &Path,
    scope: ApplicationScope,
    home: &Path,
) -> Result<InventoryEntry, CommandError> {
    let identity = application_identity(path)?;
    let (logical_size, allocated_size) = measure_bundle(&identity.canonical_path);
    let metadata = read_bundle_metadata(&identity.canonical_path);
    let fallback_name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Application")
        .to_owned();
    let name = metadata.name.unwrap_or(fallback_name);
    let self_app = metadata.bundle_id.as_deref() == Some(DISKSAGE_BUNDLE_ID);
    let uninstall_allowed = scope != ApplicationScope::System && !self_app;
    let uninstall_block_reason = if scope == ApplicationScope::System {
        Some("macOS system applications are protected and are list-only.".to_owned())
    } else if self_app {
        Some("DiskSage cannot move itself to Trash while it is running.".to_owned())
    } else {
        None
    };
    let id = blake3::hash(identity.canonical_path.to_string_lossy().as_bytes())
        .to_hex()
        .to_string();
    let display_path = display_path(&identity.canonical_path, home);
    Ok(InventoryEntry {
        application: InstalledApplication {
            id,
            name,
            bundle_id: metadata.bundle_id,
            version: metadata.version,
            path: identity.canonical_path.to_string_lossy().into_owned(),
            display_path,
            logical_size,
            allocated_size: Some(allocated_size),
            last_used_at: last_used_at(&identity.canonical_path),
            scope,
            uninstall_allowed,
            uninstall_block_reason,
        },
        identity,
    })
}

fn discover_related_items(
    application: &InstalledApplication,
    home: &Path,
    include_ambiguous: bool,
) -> Vec<PlannedRelatedItem> {
    let library = home.join("Library");
    let mut candidates = Vec::new();

    if valid_path_component(&application.name) {
        candidates.extend([
            RelatedCandidate::identified(
                library.join("Application Support").join(&application.name),
                library.clone(),
                "Application Support",
                true,
                "Exact app-name match in a known Library location.",
            ),
            RelatedCandidate::identified(
                library.join("Caches").join(&application.name),
                library.clone(),
                "Cache",
                false,
                "Exact app-name match in a known Library location.",
            ),
            RelatedCandidate::identified(
                library.join("Logs").join(&application.name),
                library.clone(),
                "Logs",
                false,
                "Exact app-name match in a known Library location.",
            ),
        ]);
    }
    if let Some(bundle_id) = application
        .bundle_id
        .as_deref()
        .filter(|value| valid_path_component(value))
    {
        candidates.extend([
            RelatedCandidate::identified(
                library.join("Application Support").join(bundle_id),
                library.clone(),
                "Application Support",
                true,
                "Exact bundle-identifier match in a known Library location.",
            ),
            RelatedCandidate::identified(
                library.join("Caches").join(bundle_id),
                library.clone(),
                "Cache",
                false,
                "Exact bundle-identifier match in a known Library location.",
            ),
            RelatedCandidate::identified(
                library
                    .join("Preferences")
                    .join(format!("{bundle_id}.plist")),
                library.clone(),
                "Preferences",
                false,
                "Exact bundle-identifier preference file.",
            ),
            RelatedCandidate::identified(
                library
                    .join("Saved Application State")
                    .join(format!("{bundle_id}.savedState")),
                library.clone(),
                "Saved application state",
                false,
                "Exact bundle-identifier saved state.",
            ),
            RelatedCandidate::identified(
                library.join("Logs").join(bundle_id),
                library.clone(),
                "Logs",
                false,
                "Exact bundle-identifier match in a known Library location.",
            ),
            RelatedCandidate::identified(
                library.join("WebKit").join(bundle_id),
                library.clone(),
                "Web data",
                true,
                "Exact bundle-identifier web storage.",
            ),
            RelatedCandidate::identified(
                library.join("HTTPStorages").join(bundle_id),
                library.clone(),
                "HTTP storage",
                false,
                "Exact bundle-identifier HTTP storage.",
            ),
            RelatedCandidate::identified(
                library.join("Containers").join(bundle_id),
                library.clone(),
                "Sandbox container",
                true,
                "Exact bundle-identifier sandbox container.",
            ),
            RelatedCandidate::identified(
                library.join("Application Scripts").join(bundle_id),
                library.clone(),
                "Application scripts",
                false,
                "Exact bundle-identifier application scripts.",
            ),
            RelatedCandidate::identified(
                library
                    .join("Cookies")
                    .join(format!("{bundle_id}.binarycookies")),
                library.clone(),
                "Cookies",
                false,
                "Exact bundle-identifier cookie store.",
            ),
        ]);
    }

    if include_ambiguous {
        let documents = home.join("Documents");
        if valid_path_component(&application.name) {
            candidates.push(RelatedCandidate::ambiguous(
                documents.join(&application.name),
                documents,
                "Documents folder",
                true,
                "Exact app-name folder in Documents; it may contain user-created work.",
            ));
        }
        let group_root = library.join("Group Containers");
        let mut group_ids = application_groups(Path::new(&application.path));
        if let Some(bundle_id) = application
            .bundle_id
            .as_deref()
            .filter(|value| valid_path_component(value))
        {
            group_ids.push(bundle_id.to_owned());
        }
        group_ids.sort();
        group_ids.dedup();
        for group_id in group_ids
            .into_iter()
            .filter(|value| valid_path_component(value))
        {
            candidates.push(RelatedCandidate::ambiguous(
                group_root.join(&group_id),
                group_root.clone(),
                "Shared Group Container",
                true,
                "Declared or bundle-matched shared container; other apps may use it.",
            ));
        }
    }

    let mut seen = HashSet::new();
    let mut related_items = Vec::new();
    for candidate in candidates {
        let canonical_root = candidate
            .root
            .canonicalize()
            .unwrap_or_else(|_| candidate.root.clone());
        let Ok(identity) = related_identity(&candidate.path, &canonical_root) else {
            continue;
        };
        if !seen.insert(identity.canonical_path.clone()) {
            continue;
        }
        let (logical_size, allocated_size) = measure_bundle(&identity.canonical_path);
        let id = blake3::hash(identity.canonical_path.to_string_lossy().as_bytes())
            .to_hex()
            .to_string();
        related_items.push(PlannedRelatedItem {
            public: RelatedApplicationItem {
                id,
                path: identity.canonical_path.to_string_lossy().into_owned(),
                display_path: display_path(&identity.canonical_path, home),
                category: candidate.category.to_owned(),
                logical_size,
                allocated_size: Some(allocated_size),
                may_contain_user_data: candidate.may_contain_user_data,
                confidence: candidate.confidence,
                default_selected: candidate.default_selected,
                reason: candidate.reason.to_owned(),
            },
            identity,
            canonical_root,
        });
    }
    related_items.sort_by(|left, right| {
        right
            .public
            .default_selected
            .cmp(&left.public.default_selected)
            .then_with(|| left.public.display_path.cmp(&right.public.display_path))
    });
    related_items
}

struct RelatedCandidate {
    path: PathBuf,
    root: PathBuf,
    category: &'static str,
    may_contain_user_data: bool,
    confidence: RelatedItemConfidence,
    default_selected: bool,
    reason: &'static str,
}

impl RelatedCandidate {
    fn identified(
        path: PathBuf,
        root: PathBuf,
        category: &'static str,
        may_contain_user_data: bool,
        reason: &'static str,
    ) -> Self {
        Self {
            path,
            root,
            category,
            may_contain_user_data,
            confidence: RelatedItemConfidence::Identified,
            default_selected: true,
            reason,
        }
    }

    fn ambiguous(
        path: PathBuf,
        root: PathBuf,
        category: &'static str,
        may_contain_user_data: bool,
        reason: &'static str,
    ) -> Self {
        Self {
            path,
            root,
            category,
            may_contain_user_data,
            confidence: RelatedItemConfidence::Ambiguous,
            default_selected: false,
            reason,
        }
    }
}

fn application_groups(application_path: &Path) -> Vec<String> {
    let Ok(output) = Command::new("codesign")
        .args(["-d", "--entitlements", ":-"])
        .arg(application_path)
        .output()
    else {
        return Vec::new();
    };
    for bytes in [&output.stdout, &output.stderr] {
        let text = String::from_utf8_lossy(bytes);
        let Some(start) = text.find("<plist") else {
            continue;
        };
        let Some(relative_end) = text[start..].find("</plist>") else {
            continue;
        };
        let end = start + relative_end + "</plist>".len();
        let Ok(value) = plist::Value::from_reader_xml(text[start..end].as_bytes()) else {
            continue;
        };
        if let Some(groups) = value
            .as_dictionary()
            .and_then(|dictionary| dictionary.get("com.apple.security.application-groups"))
            .and_then(plist::Value::as_array)
        {
            return groups
                .iter()
                .filter_map(plist::Value::as_string)
                .map(str::to_owned)
                .collect();
        }
    }
    Vec::new()
}

fn valid_path_component(value: &str) -> bool {
    !value.is_empty() && value != "." && value != ".." && Path::new(value).components().count() == 1
}

#[derive(Default)]
struct BundleMetadata {
    name: Option<String>,
    bundle_id: Option<String>,
    version: Option<String>,
}

fn read_bundle_metadata(path: &Path) -> BundleMetadata {
    let Ok(value) = plist::Value::from_file(path.join("Contents/Info.plist")) else {
        return BundleMetadata::default();
    };
    let Some(dictionary) = value.as_dictionary() else {
        return BundleMetadata::default();
    };
    let string = |key: &str| {
        dictionary
            .get(key)
            .and_then(plist::Value::as_string)
            .map(str::to_owned)
    };
    BundleMetadata {
        name: string("CFBundleDisplayName").or_else(|| string("CFBundleName")),
        bundle_id: string("CFBundleIdentifier"),
        version: string("CFBundleShortVersionString"),
    }
}

fn measure_bundle(root: &Path) -> (u64, u64) {
    let mut logical = 0_u64;
    let mut allocated = 0_u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_file() {
            logical = logical.saturating_add(metadata.len());
            allocated = allocated.saturating_add(allocated_bytes(&metadata));
        } else if metadata.is_dir() {
            let Ok(children) = fs::read_dir(path) else {
                continue;
            };
            stack.extend(children.flatten().map(|entry| entry.path()));
        }
    }
    (logical, allocated)
}

#[cfg(unix)]
fn allocated_bytes(metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.blocks().saturating_mul(512)
}

#[cfg(not(unix))]
fn allocated_bytes(metadata: &fs::Metadata) -> u64 {
    metadata.len()
}

fn application_identity(path: &Path) -> Result<FileIdentity, CommandError> {
    let link_metadata =
        fs::symlink_metadata(path).map_err(|error| filesystem_error(path, error))?;
    if link_metadata.file_type().is_symlink() || !link_metadata.is_dir() {
        return Err(CommandError::new(
            ErrorCode::PlanValidationFailed,
            "The application bundle changed or is no longer a directory.",
            true,
        )
        .with_path(path.to_string_lossy()));
    }
    let canonical_path = path
        .canonicalize()
        .map_err(|error| filesystem_error(path, error))?;
    if canonical_path.extension().and_then(|value| value.to_str()) != Some("app") {
        return Err(CommandError::new(
            ErrorCode::InvalidPath,
            "Only an exact .app bundle can be reviewed for uninstall.",
            false,
        )
        .with_path(path.to_string_lossy()));
    }
    identity_from_metadata(canonical_path, link_metadata)
}

fn related_identity(path: &Path, canonical_library: &Path) -> Result<FileIdentity, CommandError> {
    let link_metadata =
        fs::symlink_metadata(path).map_err(|error| filesystem_error(path, error))?;
    if link_metadata.file_type().is_symlink() {
        return Err(CommandError::new(
            ErrorCode::PlanValidationFailed,
            "A symbolic link cannot be included in an application uninstall plan.",
            false,
        )
        .with_path(path.to_string_lossy()));
    }
    let canonical_path = path
        .canonicalize()
        .map_err(|error| filesystem_error(path, error))?;
    if !canonical_path.starts_with(canonical_library) || canonical_path == canonical_library {
        return Err(CommandError::new(
            ErrorCode::PathProtected,
            "Related application data moved outside its reviewed allowlisted folder.",
            false,
        )
        .with_path(path.to_string_lossy()));
    }
    identity_from_metadata(canonical_path, link_metadata)
}

fn identity_from_metadata(
    canonical_path: PathBuf,
    metadata: fs::Metadata,
) -> Result<FileIdentity, CommandError> {
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    Ok(FileIdentity {
        canonical_path,
        is_directory: metadata.is_dir(),
        modified_at: metadata.modified().ok(),
        #[cfg(unix)]
        device: metadata.dev(),
        #[cfg(unix)]
        inode: metadata.ino(),
    })
}

fn revalidate_application_identity(expected: &FileIdentity) -> Result<(), CommandError> {
    let current = application_identity(&expected.canonical_path)?;
    if &current != expected {
        return Err(CommandError::new(
            ErrorCode::PlanValidationFailed,
            "The application changed after review. Scan applications and create a new plan.",
            true,
        )
        .with_path(expected.canonical_path.to_string_lossy()));
    }
    Ok(())
}

fn revalidate_related_identity(expected: &PlannedRelatedItem) -> Result<(), CommandError> {
    let current = related_identity(&expected.identity.canonical_path, &expected.canonical_root)?;
    if current != expected.identity {
        return Err(CommandError::new(
            ErrorCode::PlanValidationFailed,
            "Related application data changed after review. Create a new uninstall plan.",
            true,
        )
        .with_path(expected.identity.canonical_path.to_string_lossy()));
    }
    Ok(())
}

fn is_allowed_application_path(path: &Path) -> bool {
    path.starts_with("/Applications") && !path.starts_with("/System")
        || path
            .components()
            .any(|component| component.as_os_str() == "Applications")
            && !path.starts_with("/System")
}

fn display_path(path: &Path, home: &Path) -> String {
    path.strip_prefix(home)
        .map(|relative| format!("~/{}", relative.to_string_lossy()))
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

fn filesystem_error(path: &Path, error: std::io::Error) -> CommandError {
    CommandError::new(
        if error.kind() == std::io::ErrorKind::NotFound {
            ErrorCode::PathNotFound
        } else if error.kind() == std::io::ErrorKind::PermissionDenied {
            ErrorCode::PermissionDenied
        } else {
            ErrorCode::FilesystemError
        },
        "The application bundle could not be inspected.",
        true,
    )
    .with_path(path.to_string_lossy())
    .with_details(error.to_string())
}

#[cfg(target_os = "macos")]
fn last_used_at(path: &Path) -> Option<DateTime<Utc>> {
    let output = Command::new("mdls")
        .args(["-raw", "-name", "kMDItemLastUsedDate"])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    if value == "(null)" {
        return None;
    }
    DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S %z")
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

#[cfg(not(target_os = "macos"))]
fn last_used_at(_path: &Path) -> Option<DateTime<Utc>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn fixture_app(root: &Path, name: &str, bundle_id: &str) -> PathBuf {
        let app = root.join(format!("{name}.app"));
        fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
        let mut info = fs::File::create(app.join("Contents/Info.plist")).unwrap();
        write!(info, "<?xml version=\"1.0\" encoding=\"UTF-8\"?><plist version=\"1.0\"><dict><key>CFBundleDisplayName</key><string>{name}</string><key>CFBundleIdentifier</key><string>{bundle_id}</string><key>CFBundleShortVersionString</key><string>1.2.3</string></dict></plist>").unwrap();
        fs::write(app.join("Contents/MacOS/binary"), b"fixture").unwrap();
        app
    }

    #[test]
    fn system_roots_are_opt_in() {
        let home = Path::new("/Users/fixture");
        let default_roots = application_roots(home, false);
        assert!(default_roots
            .iter()
            .all(|(_, scope)| *scope != ApplicationScope::System));

        let expanded_roots = application_roots(home, true);
        assert!(expanded_roots
            .iter()
            .any(|(path, scope)| path == Path::new("/System/Applications")
                && *scope == ApplicationScope::System));
        assert!(expanded_roots.iter().any(|(path, scope)| path
            == Path::new("/System/Library/CoreServices/Applications")
            && *scope == ApplicationScope::System));
    }

    #[test]
    fn running_process_detection_is_bound_to_the_reviewed_bundle() {
        let processes = "/Applications/Fixture.app/Contents/MacOS/Fixture --restored\n/Applications/Other.app/Contents/MacOS/Other\n";
        assert!(process_list_contains_bundle(
            processes,
            Path::new("/Applications/Fixture.app")
        ));
        assert!(!process_list_contains_bundle(
            processes,
            Path::new("/Applications/Missing.app")
        ));
    }

    #[test]
    fn inspects_bundle_metadata_and_size() {
        let dir = tempfile::tempdir().unwrap();
        let app = fixture_app(dir.path(), "Fixture", "com.example.fixture");
        let entry = inspect_application(&app, ApplicationScope::User, dir.path()).unwrap();
        assert_eq!(entry.application.name, "Fixture");
        assert_eq!(entry.application.version.as_deref(), Some("1.2.3"));
        assert!(entry.application.logical_size > 7);
        assert!(entry.application.uninstall_allowed);
    }

    #[test]
    fn blocks_system_and_disksage_bundles() {
        let dir = tempfile::tempdir().unwrap();
        let system = fixture_app(dir.path(), "SystemFixture", "com.apple.fixture");
        let disksage = fixture_app(dir.path(), "DiskSage", DISKSAGE_BUNDLE_ID);
        assert!(
            !inspect_application(&system, ApplicationScope::System, dir.path())
                .unwrap()
                .application
                .uninstall_allowed
        );
        assert!(
            !inspect_application(&disksage, ApplicationScope::User, dir.path())
                .unwrap()
                .application
                .uninstall_allowed
        );
    }

    #[test]
    fn plan_is_inventory_bound_and_single_use() {
        let dir = tempfile::tempdir().unwrap();
        let app = fixture_app(dir.path(), "Fixture", "com.example.fixture");
        let entry = inspect_application(&app, ApplicationScope::User, dir.path()).unwrap();
        let manager = ApplicationManager::default();
        manager
            .inventory
            .lock()
            .unwrap()
            .insert(entry.application.id.clone(), entry.clone());
        assert_eq!(
            manager
                .create_plan(
                    "not-in-inventory",
                    ApplicationUninstallMode::AppOnly,
                    dir.path()
                )
                .unwrap_err()
                .code,
            ErrorCode::PlanValidationFailed
        );
        let plan = manager
            .create_plan(
                &entry.application.id,
                ApplicationUninstallMode::AppOnly,
                dir.path(),
            )
            .unwrap();
        let request = ExecuteApplicationUninstallRequest {
            plan_id: plan.id,
            confirmation_token: plan.confirmation_token,
            selected_related_item_ids: Vec::new(),
            typed_confirmation: None,
        };
        manager.consume_plan(&request).unwrap();
        assert_eq!(
            manager.consume_plan(&request).unwrap_err().code,
            ErrorCode::PlanValidationFailed
        );
    }

    #[test]
    fn plan_fails_when_bundle_changes_after_review() {
        let dir = tempfile::tempdir().unwrap();
        let app = fixture_app(dir.path(), "Fixture", "com.example.fixture");
        let entry = inspect_application(&app, ApplicationScope::User, dir.path()).unwrap();
        fs::rename(&app, dir.path().join("Moved.app")).unwrap();
        assert_eq!(
            revalidate_application_identity(&entry.identity)
                .unwrap_err()
                .code,
            ErrorCode::PathNotFound
        );
    }

    #[test]
    fn complete_plan_lists_only_exact_allowlisted_related_items() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let app = fixture_app(dir.path(), "Fixture", "com.example.fixture");
        let cache = home.join("Library/Caches/com.example.fixture");
        let support = home.join("Library/Application Support/Fixture");
        let container = home.join("Library/Containers/com.example.fixture");
        let shared = home.join("Library/Group Containers/com.example.fixture");
        let documents = home.join("Documents/Fixture");
        let unrelated = home.join("Library/Caches/com.example.other");
        for path in [
            &cache, &support, &container, &shared, &documents, &unrelated,
        ] {
            fs::create_dir_all(path).unwrap();
            fs::write(path.join("fixture"), b"data").unwrap();
        }

        let entry = inspect_application(&app, ApplicationScope::User, &home).unwrap();
        let manager = ApplicationManager::default();
        manager
            .inventory
            .lock()
            .unwrap()
            .insert(entry.application.id.clone(), entry.clone());

        let app_only = manager
            .create_plan(
                &entry.application.id,
                ApplicationUninstallMode::AppOnly,
                &home,
            )
            .unwrap();
        assert!(app_only.related_items.is_empty());

        let complete = manager
            .create_plan(
                &entry.application.id,
                ApplicationUninstallMode::Complete,
                &home,
            )
            .unwrap();
        let paths: HashSet<_> = complete
            .related_items
            .iter()
            .map(|item| item.path.as_str())
            .collect();
        assert_eq!(paths.len(), 3);
        let canonical = |path: &Path| path.canonicalize().unwrap().to_string_lossy().into_owned();
        assert!(paths.contains(canonical(&cache).as_str()));
        assert!(paths.contains(canonical(&support).as_str()));
        assert!(paths.contains(canonical(&container).as_str()));
        assert!(!paths.contains(canonical(&shared).as_str()));
        assert!(!paths.contains(canonical(&unrelated).as_str()));
        assert!(complete
            .related_items
            .iter()
            .find(|item| item.category == "Sandbox container")
            .is_some_and(|item| item.may_contain_user_data));
        assert!(complete.total_expected_bytes > app_only.total_expected_bytes);

        let deep = manager
            .create_plan(
                &entry.application.id,
                ApplicationUninstallMode::DeepCleanup,
                &home,
            )
            .unwrap();
        assert_eq!(
            deep.required_confirmation_phrase.as_deref(),
            Some("DEEP CLEAN Fixture")
        );
        let shared_item = deep
            .related_items
            .iter()
            .find(|item| item.path == canonical(&shared))
            .unwrap();
        assert_eq!(shared_item.confidence, RelatedItemConfidence::Ambiguous);
        assert!(!shared_item.default_selected);
        let shared_item_id = shared_item.id.clone();
        let document_item = deep
            .related_items
            .iter()
            .find(|item| item.path == canonical(&documents))
            .unwrap();
        assert_eq!(document_item.confidence, RelatedItemConfidence::Ambiguous);
        assert!(!document_item.default_selected);
        let document_item_id = document_item.id.clone();

        let selected_related_item_ids: Vec<_> = deep
            .related_items
            .iter()
            .filter(|item| item.default_selected || item.id == shared_item_id)
            .map(|item| item.id.clone())
            .collect();
        let mut request = ExecuteApplicationUninstallRequest {
            plan_id: deep.id,
            confirmation_token: deep.confirmation_token,
            selected_related_item_ids,
            typed_confirmation: Some("wrong".to_owned()),
        };
        assert_eq!(
            manager.consume_plan(&request).unwrap_err().code,
            ErrorCode::PlanValidationFailed
        );
        request.typed_confirmation = Some("DEEP CLEAN Fixture".to_owned());
        let validated = manager.consume_plan(&request).unwrap();
        assert!(validated
            .related_items
            .iter()
            .any(|item| item.public.id == shared_item_id));
        assert!(!validated
            .related_items
            .iter()
            .any(|item| item.public.id == document_item_id));
    }

    #[test]
    fn malformed_bundle_names_cannot_escape_the_library_allowlist() {
        assert!(!valid_path_component("../Documents"));
        assert!(!valid_path_component("a/b"));
        assert!(!valid_path_component(".."));
        assert!(valid_path_component("com.example.safe"));
    }
}
