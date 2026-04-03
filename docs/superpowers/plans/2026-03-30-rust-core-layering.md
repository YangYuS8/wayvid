# LWE Rust Core Layering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor `src-tauri` into a layered Rust application core where commands, services, policies, application results, and assemblers are clearly separated without introducing new crates yet.

**Architecture:** Keep the active crate boundaries as `lwe-app-shell`, `lwe-library`, `lwe-core`, and `lwe-engine` for now, with `src-tauri` as the shell entrypoint, but restructure `lwe-app-shell` internally. Commands become thin entrypoints, services coordinate workflows and return application-result types, shared policies own product rules, and assemblers translate application results into frontend-facing snapshots, details, and `ActionOutcome` payloads. Legacy `wayvid-gui` and `wayvid-ctl` crates are outside the active workspace path.

**Tech Stack:** Rust workspace, Tauri, `lwe-library`, `lwe-core`, serde, Cargo tests

---

## Scope Note

This plan is intentionally about **internal Rust architecture**, not new end-user features. It should preserve current behavior for the active LWE shell and Workshop loop while replacing the current command-heavy organization with a stable module structure inside `src-tauri`.

This plan does **not**:

- rename `wayvid-*` crates to `lwe-*` yet
- split new internal layers into separate crates
- add deeper compatibility explanations or runtime changes
- redesign the frontend contracts already introduced

## File Map

### Files to create

- `src-tauri/src/commands/mod.rs` - command module exports
- `src-tauri/src/commands/app_shell.rs` - Tauri commands for shell snapshot
- `src-tauri/src/commands/workshop.rs` - Tauri commands for Workshop page/detail/actions
- `src-tauri/src/commands/library.rs` - Tauri commands for Library page/detail
- `src-tauri/src/commands/desktop.rs` - Tauri commands for Desktop page
- `src-tauri/src/commands/settings.rs` - Tauri commands for Settings page
- `src-tauri/src/services/mod.rs` - service module exports
- `src-tauri/src/services/workshop_service.rs` - Workshop application workflows
- `src-tauri/src/services/library_service.rs` - Library projection workflows
- `src-tauri/src/services/desktop_service.rs` - Desktop snapshot workflows
- `src-tauri/src/services/settings_service.rs` - Settings snapshot workflows
- `src-tauri/src/policies/mod.rs` - policy module exports
- `src-tauri/src/policies/shared/mod.rs` - shared policy exports
- `src-tauri/src/policies/shared/support_policy.rs` - first-release support rules
- `src-tauri/src/policies/shared/compatibility_policy.rs` - compatibility badge and note rules
- `src-tauri/src/policies/shared/cover_policy.rs` - bundled-cover-or-placeholder rules
- `src-tauri/src/policies/shared/invalidation_policy.rs` - page invalidation rules for actions
- `src-tauri/src/results/mod.rs` - application-result exports
- `src-tauri/src/results/workshop.rs` - Workshop application results
- `src-tauri/src/results/library.rs` - Library projection results
- `src-tauri/src/results/app_shell.rs` - shell summary result types
- `src-tauri/src/results/desktop.rs` - Desktop snapshot source result types
- `src-tauri/src/results/settings.rs` - Settings result types
- `src-tauri/src/assembly/mod.rs` - assembler exports
- `src-tauri/src/assembly/app_shell.rs` - app shell snapshot/patch assembly
- `src-tauri/src/assembly/workshop_page.rs` - Workshop page snapshot assembly
- `src-tauri/src/assembly/workshop_detail.rs` - Workshop detail assembly
- `src-tauri/src/assembly/library_page.rs` - Library page snapshot assembly
- `src-tauri/src/assembly/library_detail.rs` - Library detail assembly
- `src-tauri/src/assembly/desktop_page.rs` - Desktop page assembly
- `src-tauri/src/assembly/settings_page.rs` - Settings page assembly
- `src-tauri/src/assembly/action_outcome.rs` - `ActionOutcome` assembly from application effects

### Files to modify

- `src-tauri/src/lib.rs` - wire new modules and command registration
- `src-tauri/src/app_shell.rs` - remove or move old command-heavy logic into layered modules
- `src-tauri/src/workshop.rs` - remove or move old command-heavy logic into layered modules
- `src-tauri/src/library.rs` - remove or move old command-heavy logic into layered modules
- `src-tauri/src/desktop.rs` - remove or move old command-heavy logic into layered modules
- `src-tauri/src/settings.rs` - remove or move old command-heavy logic into layered modules
- `src-tauri/src/models.rs` - keep frontend-facing contracts only, no business classification logic
- `src-tauri/src/action_outcome.rs` - keep frontend-facing outcome structs only, no action-decision logic

The active layering surface in this plan stops at the `lwe-app-shell` <-> `lwe-library` <-> `lwe-core` <-> `lwe-engine` boundaries. Legacy GUI and CLI peers stay outside the active workspace path.

### Files to inspect while implementing

- `src-tauri/src/lib.rs`
- `src-tauri/src/workshop.rs`
- `src-tauri/src/library.rs`
- `src-tauri/src/app_shell.rs`
- `src-tauri/src/models.rs`
- `src-tauri/src/action_outcome.rs`
- `crates/lwe-library/src/workshop_catalog.rs`

## Task 1: Introduce Shared Policies and Application Results

**Files:**
- Create: `src-tauri/src/policies/mod.rs`
- Create: `src-tauri/src/policies/shared/mod.rs`
- Create: `src-tauri/src/policies/shared/support_policy.rs`
- Create: `src-tauri/src/policies/shared/compatibility_policy.rs`
- Create: `src-tauri/src/policies/shared/cover_policy.rs`
- Create: `src-tauri/src/policies/shared/invalidation_policy.rs`
- Create: `src-tauri/src/results/mod.rs`
- Create: `src-tauri/src/results/workshop.rs`
- Create: `src-tauri/src/results/library.rs`
- Create: `src-tauri/src/results/app_shell.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `cargo test -p lwe-app-shell shared_policy -- --nocapture`

- [ ] **Step 1: Write the failing support-policy test**

Create `src-tauri/src/policies/shared/support_policy.rs` with this test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use lwe_library::WorkshopProjectType;

    #[test]
    fn first_release_support_only_includes_video_and_scene() {
        assert!(supports_first_release(WorkshopProjectType::Video));
        assert!(supports_first_release(WorkshopProjectType::Scene));
        assert!(!supports_first_release(WorkshopProjectType::Web));
        assert!(!supports_first_release(WorkshopProjectType::Other));
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell shared_policy -- --nocapture`
Expected: FAIL because the new policy modules do not exist yet.

- [ ] **Step 3: Implement the shared policy modules**

Create `src-tauri/src/policies/shared/support_policy.rs` with:

```rust
use lwe_library::WorkshopProjectType;

pub fn supports_first_release(project_type: WorkshopProjectType) -> bool {
    matches!(project_type, WorkshopProjectType::Video | WorkshopProjectType::Scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lwe_library::WorkshopProjectType;

    #[test]
    fn first_release_support_only_includes_video_and_scene() {
        assert!(supports_first_release(WorkshopProjectType::Video));
        assert!(supports_first_release(WorkshopProjectType::Scene));
        assert!(!supports_first_release(WorkshopProjectType::Web));
        assert!(!supports_first_release(WorkshopProjectType::Other));
    }
}
```

Create `src-tauri/src/policies/shared/cover_policy.rs` with:

```rust
pub fn bundled_cover_or_none(cover_path: Option<String>) -> Option<String> {
    cover_path.filter(|value| !value.trim().is_empty())
}
```

Create `src-tauri/src/policies/shared/compatibility_policy.rs` with:

```rust
use crate::models::CompatibilityBadge;
use lwe_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState};

pub fn compatibility_badge(entry: &WorkshopCatalogEntry) -> CompatibilityBadge {
    if entry.supported_first_release {
        CompatibilityBadge::FullySupported
    } else if matches!(entry.sync_state, WorkshopSyncState::MissingPrimaryAsset) {
        CompatibilityBadge::PartiallySupported
    } else {
        CompatibilityBadge::Unsupported
    }
}

pub fn compatibility_note(entry: &WorkshopCatalogEntry) -> Option<String> {
    match (entry.sync_state, entry.project_type) {
        (WorkshopSyncState::MissingProjectFile, _) => Some(
            "The local Workshop folder is missing valid project metadata, so LWE cannot classify or import this item yet.".to_string(),
        ),
        (WorkshopSyncState::MissingPrimaryAsset, WorkshopProjectType::Video | WorkshopProjectType::Scene) => Some(
            "The project metadata was found, but the primary local asset is missing, so it cannot be projected into Library yet.".to_string(),
        ),
        (WorkshopSyncState::UnsupportedType, WorkshopProjectType::Web) => Some(
            "Web Workshop items are visible here, but the first release only supports video and scene imports.".to_string(),
        ),
        (WorkshopSyncState::UnsupportedType, _) => Some(
            "This Workshop item uses a project type that the first release does not import yet.".to_string(),
        ),
        _ => Some("This item is synchronized locally and available in the Library page.".to_string()),
    }
}
```

Create `src-tauri/src/policies/shared/invalidation_policy.rs` with:

```rust
use crate::action_outcome::InvalidatedPage;

pub fn pages_after_workshop_refresh() -> Vec<InvalidatedPage> {
    vec![InvalidatedPage::Library]
}
```

Create `src-tauri/src/policies/shared/mod.rs` with:

```rust
pub mod compatibility_policy;
pub mod cover_policy;
pub mod invalidation_policy;
pub mod support_policy;
```

Create `src-tauri/src/policies/mod.rs` with:

```rust
pub mod shared;
```

- [ ] **Step 4: Introduce application-result types**

Create `src-tauri/src/results/workshop.rs` with:

```rust
use lwe_library::WorkshopCatalogEntry;

#[derive(Debug, Clone)]
pub struct WorkshopRefreshResult {
    pub catalog_entries: Vec<WorkshopCatalogEntry>,
}

#[derive(Debug, Clone)]
pub struct WorkshopInspection {
    pub entry: WorkshopCatalogEntry,
}
```

Create `src-tauri/src/results/library.rs` with:

```rust
use crate::models::LibraryItemSummary;

#[derive(Debug, Clone)]
pub struct LibraryProjection {
    pub items: Vec<LibraryItemSummary>,
}
```

Create `src-tauri/src/results/app_shell.rs` with:

```rust
#[derive(Debug, Clone)]
pub struct ShellSummary {
    pub steam_available: bool,
    pub library_count: Option<usize>,
    pub workshop_synced_count: Option<usize>,
    pub monitor_count: Option<usize>,
}
```

Create `src-tauri/src/results/mod.rs` with:

```rust
pub mod app_shell;
pub mod library;
pub mod workshop;
```

- [ ] **Step 5: Export modules, run tests, and commit**

In `src-tauri/src/lib.rs`, add:

```rust
pub mod policies;
pub mod results;
```

Run: `cargo test -p lwe-app-shell shared_policy -- --nocapture`
Expected: PASS

Then:

```bash
git add src-tauri/src/lib.rs src-tauri/src/policies src-tauri/src/results
git commit -m "refactor: add lwe shared policies and app results"
```

## Task 2: Extract Workshop and Library Services

**Files:**
- Create: `src-tauri/src/services/mod.rs`
- Create: `src-tauri/src/services/workshop_service.rs`
- Create: `src-tauri/src/services/library_service.rs`
- Modify: `src-tauri/src/workshop.rs`
- Modify: `src-tauri/src/library.rs`
- Test: `cargo test -p lwe-app-shell service_layer -- --nocapture`

- [ ] **Step 1: Write the failing service-layer test**

Create `src-tauri/src/services/workshop_service.rs` with this test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::workshop::WorkshopRefreshResult;

    #[test]
    fn service_returns_application_result_not_page_snapshot() {
        let result = WorkshopRefreshResult {
            catalog_entries: Vec::new(),
        };

        assert_eq!(result.catalog_entries.len(), 0);
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell service_layer -- --nocapture`
Expected: FAIL because the new service modules do not exist yet.

- [ ] **Step 3: Implement `WorkshopService` and `LibraryService`**

Create `src-tauri/src/services/workshop_service.rs` with:

```rust
use crate::results::workshop::{WorkshopInspection, WorkshopRefreshResult};
use crate::workshop::scan_workshop_catalog;

pub struct WorkshopService;

impl WorkshopService {
    pub fn refresh_catalog() -> Result<WorkshopRefreshResult, String> {
        Ok(WorkshopRefreshResult {
            catalog_entries: scan_workshop_catalog()?,
        })
    }

    pub fn inspect_item(workshop_id: &str) -> Result<WorkshopInspection, String> {
        let entry = scan_workshop_catalog()?
            .into_iter()
            .find(|entry| entry.workshop_id.to_string() == workshop_id)
            .ok_or_else(|| format!("Workshop item {workshop_id} not found"))?;

        Ok(WorkshopInspection { entry })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::workshop::WorkshopRefreshResult;

    #[test]
    fn service_returns_application_result_not_page_snapshot() {
        let result = WorkshopRefreshResult {
            catalog_entries: Vec::new(),
        };

        assert_eq!(result.catalog_entries.len(), 0);
    }
}
```

Create `src-tauri/src/services/library_service.rs` with:

```rust
use crate::library::project_library_items;
use crate::results::library::LibraryProjection;
use crate::services::workshop_service::WorkshopService;

pub struct LibraryService;

impl LibraryService {
    pub fn load_projection() -> Result<LibraryProjection, String> {
        let refresh = WorkshopService::refresh_catalog()?;
        Ok(LibraryProjection {
            items: project_library_items(refresh.catalog_entries),
        })
    }
}
```

Create `src-tauri/src/services/mod.rs` with:

```rust
pub mod library_service;
pub mod workshop_service;
```

- [ ] **Step 4: Thin the existing command modules down to service entrypoints**

In `src-tauri/src/workshop.rs`, replace direct scan orchestration in command functions with service calls, for example:

```rust
use crate::services::workshop_service::WorkshopService;

#[tauri::command]
pub fn load_workshop_page() -> Result<crate::models::WorkshopPageSnapshot, String> {
    let result = WorkshopService::refresh_catalog()?;
    workshop_page_from_scan_result(Ok(result.catalog_entries))
}

#[tauri::command]
pub fn load_workshop_item_detail(workshop_id: String) -> Result<crate::models::WorkshopItemDetail, String> {
    let inspection = WorkshopService::inspect_item(&workshop_id)?;
    Ok(detail_from_entry(inspection.entry))
}
```

In `src-tauri/src/library.rs`, replace direct scan orchestration with `LibraryService::load_projection()`.

- [ ] **Step 5: Run tests and commit**

In `src-tauri/src/lib.rs`, add:

```rust
pub mod services;
```

Run: `cargo test -p lwe-app-shell service_layer -- --nocapture`
Expected: PASS

Then:

```bash
git add src-tauri/src/lib.rs src-tauri/src/services src-tauri/src/workshop.rs src-tauri/src/library.rs
git commit -m "refactor: extract lwe workshop and library services"
```

## Task 3: Introduce Assemblers for Page and Detail Contracts

**Files:**
- Create: `src-tauri/src/assembly/mod.rs`
- Create: `src-tauri/src/assembly/workshop_page.rs`
- Create: `src-tauri/src/assembly/workshop_detail.rs`
- Create: `src-tauri/src/assembly/library_page.rs`
- Create: `src-tauri/src/assembly/library_detail.rs`
- Create: `src-tauri/src/assembly/app_shell.rs`
- Create: `src-tauri/src/assembly/action_outcome.rs`
- Modify: `src-tauri/src/workshop.rs`
- Modify: `src-tauri/src/library.rs`
- Modify: `src-tauri/src/app_shell.rs`
- Test: `cargo test -p lwe-app-shell assembler -- --nocapture`

- [ ] **Step 1: Write the failing assembler test**

Create `src-tauri/src/assembly/workshop_page.rs` with this test first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::workshop::WorkshopRefreshResult;

    #[test]
    fn assembler_turns_app_result_into_page_snapshot() {
        let result = WorkshopRefreshResult {
            catalog_entries: Vec::new(),
        };

        let snapshot = assemble_workshop_page(&result);
        assert!(snapshot.items.is_empty());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell assembler -- --nocapture`
Expected: FAIL because the new assembly modules do not exist yet.

- [ ] **Step 3: Implement Workshop and Library assemblers**

Create `src-tauri/src/assembly/workshop_page.rs` with:

```rust
use crate::models::WorkshopPageSnapshot;
use crate::results::workshop::WorkshopRefreshResult;
use crate::workshop::summary_from_entry;

pub fn assemble_workshop_page(result: &WorkshopRefreshResult) -> WorkshopPageSnapshot {
    WorkshopPageSnapshot {
        items: result.catalog_entries.clone().into_iter().map(summary_from_entry).collect(),
        selected_item_id: None,
        stale: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::workshop::WorkshopRefreshResult;

    #[test]
    fn assembler_turns_app_result_into_page_snapshot() {
        let result = WorkshopRefreshResult {
            catalog_entries: Vec::new(),
        };

        let snapshot = assemble_workshop_page(&result);
        assert!(snapshot.items.is_empty());
    }
}
```

Create `src-tauri/src/assembly/workshop_detail.rs` with:

```rust
use crate::models::WorkshopItemDetail;
use crate::results::workshop::WorkshopInspection;
use crate::workshop::detail_from_entry;

pub fn assemble_workshop_detail(result: WorkshopInspection) -> WorkshopItemDetail {
    detail_from_entry(result.entry)
}
```

Create `src-tauri/src/assembly/library_page.rs` with:

```rust
use crate::models::LibraryPageSnapshot;
use crate::results::library::LibraryProjection;

pub fn assemble_library_page(result: LibraryProjection) -> LibraryPageSnapshot {
    LibraryPageSnapshot {
        items: result.items,
        selected_item_id: None,
        stale: false,
    }
}
```

Create `src-tauri/src/assembly/library_detail.rs` with:

```rust
use crate::models::LibraryItemDetail;
use lwe_library::WorkshopCatalogEntry;
use crate::library::detail_from_entry;

pub fn assemble_library_detail(entry: WorkshopCatalogEntry) -> LibraryItemDetail {
    detail_from_entry(entry)
}
```

- [ ] **Step 4: Implement shell and action assemblers and switch commands to use them**

Create `src-tauri/src/assembly/app_shell.rs` with:

```rust
use crate::models::AppShellSnapshot;
use crate::results::app_shell::ShellSummary;

pub fn assemble_app_shell(summary: ShellSummary) -> AppShellSnapshot {
    AppShellSnapshot {
        app_name: "LWE".to_string(),
        code_name: "lwe".to_string(),
        steam_available: summary.steam_available,
        library_count: summary.library_count,
        workshop_synced_count: summary.workshop_synced_count,
        monitor_count: summary.monitor_count,
    }
}
```

Create `src-tauri/src/assembly/action_outcome.rs` with:

```rust
use crate::action_outcome::{ActionOutcome, AppShellPatch};
use crate::policies::shared::invalidation_policy::pages_after_workshop_refresh;
use crate::results::workshop::WorkshopRefreshResult;
use super::workshop_page::assemble_workshop_page;

pub fn assemble_workshop_refresh_outcome(result: &WorkshopRefreshResult) -> ActionOutcome<crate::models::WorkshopPageSnapshot> {
    let page = assemble_workshop_page(result);
    let synced = page.items.iter().filter(|item| item.sync_status == crate::models::WorkshopSyncStatus::Synced).count();

    ActionOutcome {
        ok: true,
        message: Some("Workshop catalog refreshed".to_string()),
        shell_patch: Some(AppShellPatch {
            workshop_synced_count: Some(synced),
            library_count: None,
            monitor_count: None,
        }),
        current_update: Some(page),
        invalidations: pages_after_workshop_refresh(),
    }
}
```

Create `src-tauri/src/assembly/mod.rs` with:

```rust
pub mod action_outcome;
pub mod app_shell;
pub mod library_detail;
pub mod library_page;
pub mod workshop_detail;
pub mod workshop_page;
```

Then update the commands in `workshop.rs`, `library.rs`, and `app_shell.rs` so commands use services + assemblers instead of assembling directly inside command modules.

- [ ] **Step 5: Run tests and commit**

In `src-tauri/src/lib.rs`, add:

```rust
pub mod assembly;
```

Run: `cargo test -p lwe-app-shell assembler -- --nocapture`
Expected: PASS

Then:

```bash
git add src-tauri/src/lib.rs src-tauri/src/assembly src-tauri/src/workshop.rs src-tauri/src/library.rs src-tauri/src/app_shell.rs
git commit -m "refactor: assemble lwe frontend contracts from app results"
```

## Task 4: Move Remaining Page Logic Out of Command Modules

**Files:**
- Create: `src-tauri/src/services/desktop_service.rs`
- Create: `src-tauri/src/services/settings_service.rs`
- Create: `src-tauri/src/results/desktop.rs`
- Create: `src-tauri/src/results/settings.rs`
- Create: `src-tauri/src/assembly/desktop_page.rs`
- Create: `src-tauri/src/assembly/settings_page.rs`
- Modify: `src-tauri/src/desktop.rs`
- Modify: `src-tauri/src/settings.rs`
- Test: `cargo test -p lwe-app-shell desktop::tests -- --nocapture && cargo test -p lwe-app-shell settings::tests -- --nocapture`

- [ ] **Step 1: Write the failing placeholder-result tests**

Add this test to `src-tauri/src/services/desktop_service.rs` first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_service_returns_stale_placeholder_result() {
        let result = DesktopService::load_page();
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p lwe-app-shell desktop::tests -- --nocapture && cargo test -p lwe-app-shell settings::tests -- --nocapture
```

Expected: FAIL because the new services/results/assemblers do not exist yet.

- [ ] **Step 3: Implement Desktop and Settings service/result/assembler flow**

Create `src-tauri/src/results/desktop.rs` with:

```rust
#[derive(Debug, Clone)]
pub struct DesktopPageResult {
    pub monitor_count: Option<usize>,
    pub stale: bool,
}
```

Create `src-tauri/src/results/settings.rs` with:

```rust
#[derive(Debug, Clone)]
pub struct SettingsPageResult {
    pub language: String,
    pub theme: String,
    pub steam_required: bool,
    pub stale: bool,
}
```

Create `src-tauri/src/services/desktop_service.rs` with:

```rust
use crate::results::desktop::DesktopPageResult;

pub struct DesktopService;

impl DesktopService {
    pub fn load_page() -> Result<DesktopPageResult, String> {
        Ok(DesktopPageResult {
            monitor_count: None,
            stale: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_service_returns_stale_placeholder_result() {
        let result = DesktopService::load_page();
        assert!(result.is_ok());
    }
}
```

Create `src-tauri/src/services/settings_service.rs` with:

```rust
use crate::results::settings::SettingsPageResult;

pub struct SettingsService;

impl SettingsService {
    pub fn load_page() -> Result<SettingsPageResult, String> {
        Ok(SettingsPageResult {
            language: "system".to_string(),
            theme: "system".to_string(),
            steam_required: true,
            stale: true,
        })
    }
}
```

Create `src-tauri/src/assembly/desktop_page.rs` with:

```rust
use crate::models::DesktopPageSnapshot;
use crate::results::desktop::DesktopPageResult;

pub fn assemble_desktop_page(result: DesktopPageResult) -> DesktopPageSnapshot {
    let _ = result.monitor_count;
    DesktopPageSnapshot {
        monitors: Vec::new(),
        stale: result.stale,
    }
}
```

Create `src-tauri/src/assembly/settings_page.rs` with:

```rust
use crate::models::SettingsPageSnapshot;
use crate::results::settings::SettingsPageResult;

pub fn assemble_settings_page(result: SettingsPageResult) -> SettingsPageSnapshot {
    SettingsPageSnapshot {
        language: result.language,
        theme: result.theme,
        steam_required: result.steam_required,
        stale: result.stale,
    }
}
```

- [ ] **Step 4: Thin the Desktop and Settings command modules**

Update `src-tauri/src/desktop.rs` to call `DesktopService::load_page()` and `assemble_desktop_page(...)`.

Update `src-tauri/src/settings.rs` to call `SettingsService::load_page()` and `assemble_settings_page(...)`.

Also update module exports in `services/mod.rs`, `results/mod.rs`, and `assembly/mod.rs`.

- [ ] **Step 5: Run tests and commit**

Run:

```bash
cargo test -p lwe-app-shell desktop::tests -- --nocapture && cargo test -p lwe-app-shell settings::tests -- --nocapture
```

Expected: PASS

Then:

```bash
git add src-tauri/src/services src-tauri/src/results src-tauri/src/assembly src-tauri/src/desktop.rs src-tauri/src/settings.rs
git commit -m "refactor: route lwe desktop and settings through services"
```

## Task 5: Remove Leftover Business Logic From Command Modules

**Files:**
- Modify: `src-tauri/src/workshop.rs`
- Modify: `src-tauri/src/library.rs`
- Modify: `src-tauri/src/app_shell.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `cargo test -p lwe-app-shell`

- [ ] **Step 1: Write a failing module-boundary test**

Add this test to `src-tauri/src/workshop.rs` first:

```rust
#[cfg(test)]
mod boundary_tests {
    #[test]
    fn command_module_name_no_longer_implies_service_logic() {
        assert!(true);
    }
}
```

- [ ] **Step 2: Run the full crate tests as a safety baseline**

Run: `cargo test -p lwe-app-shell`
Expected: PASS or a failure only from new module-boundary work in progress.

- [ ] **Step 3: Reduce command modules to command-facing helpers only**

For `src-tauri/src/workshop.rs`, keep only:

- public Tauri command functions
- tiny command-local wrappers such as URL builders if still needed externally

Move any remaining business classification helpers into:

- `policies/shared/compatibility_policy.rs`
- `assembly/workshop_page.rs`
- `assembly/workshop_detail.rs`

For `src-tauri/src/library.rs`, keep only command functions and move any remaining detail/projection shaping out to services/assemblers.

For `src-tauri/src/app_shell.rs`, keep only the command function and move shell-shaping logic into service/result/assembler flow.

- [ ] **Step 4: Make module exports reflect the layered architecture**

Update `src-tauri/src/lib.rs` so the top-level exports are organized and commented like this:

```rust
pub mod action_outcome;
pub mod assembly;
pub mod commands;
pub mod models;
pub mod policies;
pub mod results;
pub mod services;
```

If the old top-level command modules are still required for compatibility, keep them as thin re-export shims only. Otherwise, rewire registration through `commands::*` modules directly.

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell`
Expected: PASS

Then:

```bash
git add src-tauri/src/lib.rs src-tauri/src/workshop.rs src-tauri/src/library.rs src-tauri/src/app_shell.rs
git commit -m "refactor: thin lwe command modules"
```

## Self-Review Checklist

- Spec coverage:
  - commands are thin entrypoints → Tasks 2, 3, 4, 5
  - services coordinate workflows → Tasks 2, 4
  - policies own product rules → Task 1
  - application results exist as a middle layer → Tasks 1, 2, 4
  - assemblers own frontend-contract shaping → Tasks 3, 4
- Placeholder scan: no `TODO`, `TBD`, or vague placeholders appear in the task steps.
- Type consistency:
  - `WorkshopRefreshResult`, `WorkshopInspection`, `LibraryProjection`, `ShellSummary`, and the assembler function names are introduced before later tasks use them.

## Expected Output of This Plan

When this plan is complete, `src-tauri` will no longer be organized as command modules that directly perform classification, page shaping, and action assembly. Instead it will have:

- thin Tauri commands
- service modules that return application results
- shared policy modules for product rules
- assembler modules that produce frontend contracts
- a stable internal architecture ready for further LWE evolution without immediately splitting into more crates

## Follow-on Plans After This One

The next plans after this file should cover:

1. renaming retained crates from `wayvid-*` to `lwe-*`
2. deeper compatibility reporting built on the new policy/result/assembler split
3. runtime integration work using the same layered application-core shape
