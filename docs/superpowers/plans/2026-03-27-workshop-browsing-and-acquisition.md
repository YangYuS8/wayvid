# LWE Workshop Browsing and Acquisition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first reset-era LWE Workshop loop with a Tauri + Svelte thin frontend, a Rust-owned Workshop page snapshot model, official Steam handoff actions, and Library projection for locally synchronized items.

**Architecture:** This plan replaces the old `wayvid-gui`/`iced` Workshop direction with the active `lwe` application shell rooted at `apps/lwe/src-tauri` and `apps/lwe`. Rust remains the product brain: it discovers Steam libraries, scans locally synchronized Wallpaper Engine items, resolves bundled covers, produces page snapshots and item details, and returns `ActionOutcome` results for state-changing commands. The Svelte frontend stays thin: it renders `Library / Workshop / Desktop / Settings`, caches page snapshots by page, requests details on selection, and obeys stale-page invalidation instead of maintaining its own business truth.

**Tech Stack:** Rust workspace, Tauri, Svelte, TypeScript, `wayvid-library`, `lwe-core`, Steam local filesystem discovery, `open` crate, Cargo tests, Vitest

---

## Scope Note

The product blueprint's Workshop track is still too large for one implementation pass. This plan covers only the first slice:

1. create the new `lwe` Tauri + Svelte shell
2. expose Rust-owned Workshop page snapshots and item details
3. hand off acquisition to official Steam entry points
4. project synchronized Workshop-supported items into the Library page

This plan does **not** implement:

- remote Workshop browsing beyond locally synchronized items
- compatibility deep-dive explanations beyond basic badges/details
- `video`/`scene` runtime execution changes
- full Rust application-core layering redesign

## File Map

### Files to create

- `apps/lwe/package.json` - frontend package manifest for the new thin frontend shell
- `apps/lwe/svelte.config.js` - Svelte app configuration
- `apps/lwe/vite.config.ts` - Vite configuration for the frontend
- `apps/lwe/tsconfig.json` - TypeScript configuration for the frontend
- `apps/lwe/src/app.d.ts` - Svelte TypeScript declarations
- `apps/lwe/src/lib/types.ts` - frontend snapshot/detail/action TypeScript types
- `apps/lwe/src/lib/ipc.ts` - thin typed wrappers over Tauri commands/events
- `apps/lwe/src/lib/stores/ui.ts` - page selection, stale flags, and current-detail UI state only
- `apps/lwe/src/routes/+layout.svelte` - global app shell for `Library / Workshop / Desktop / Settings`
- `apps/lwe/src/routes/+page.svelte` - default redirect or landing behavior to `Library`
- `apps/lwe/src/routes/library/+page.svelte` - thin Library page using Rust snapshots
- `apps/lwe/src/routes/workshop/+page.svelte` - thin Workshop page using Rust snapshots and detail fetches
- `apps/lwe/src/routes/desktop/+page.svelte` - thin Desktop page shell using Rust snapshots
- `apps/lwe/src/routes/settings/+page.svelte` - thin Settings page shell using Rust snapshots
- `apps/lwe/src/lib/components/ItemCard.svelte` - shared summary-card renderer using bundled cover or placeholder
- `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte` - Workshop detail panel component
- `apps/lwe/src/lib/components/LibraryDetailPanel.svelte` - Library detail panel component
- `apps/lwe/src/lib/components/StatusBadge.svelte` - compact badge component for sync/compatibility/source states
- `apps/lwe/src/lib/components/CoverImage.svelte` - cover-or-placeholder rendering component
- `apps/lwe/src-tauri/Cargo.toml` - Tauri Rust package manifest for the `lwe` app shell
- `apps/lwe/src-tauri/tauri.conf.json` - Tauri configuration for the `lwe` desktop shell
- `apps/lwe/src-tauri/src/main.rs` - Tauri entry point
- `apps/lwe/src-tauri/src/lib.rs` - Tauri command registration module
- `apps/lwe/src-tauri/src/models.rs` - serialized snapshot/detail/action result types for frontend commands
- `apps/lwe/src-tauri/src/app_shell.rs` - app-shell snapshot assembly and shell patch helpers
- `apps/lwe/src-tauri/src/workshop.rs` - Workshop page snapshot, detail, and action commands
- `apps/lwe/src-tauri/src/library.rs` - Library page snapshot and Workshop-to-Library projection commands
- `apps/lwe/src-tauri/src/desktop.rs` - Desktop page snapshot shell commands
- `apps/lwe/src-tauri/src/settings.rs` - Settings page snapshot shell commands
- `apps/lwe/src-tauri/src/action_outcome.rs` - shared `ActionOutcome<T>` model and invalidation enums
- `crates/wayvid-library/src/workshop_catalog.rs` - catalog/domain layer for synchronized Workshop entries, bundled covers, and first-release support state

### Files to modify

- `Cargo.toml` - keep `apps/lwe/src-tauri` as the active Rust workspace shell member
- `README.md` - add a short note that the reset-era app shell is now `LWE` / `lwe`
- `docs/product/roadmap.md` - align the Workshop planning track with the new Tauri + Svelte direction if wording still mentions only generic browsing
- `crates/wayvid-library/src/lib.rs` - export new Workshop catalog types
- `crates/wayvid-library/src/workshop.rs` - expose conversion helpers from parsed Workshop data into catalog/page inputs

### Files to inspect while implementing

- `docs/superpowers/specs/2026-03-27-linux-dynamic-wallpaper-platform-design.md`
- `docs/product/overview.md`
- `docs/product/roadmap.md`
- `crates/wayvid-library/src/workshop.rs`
- `crates/lwe-core/src/library.rs`
- `crates/wayvid-gui/locales/en.toml`
- `crates/wayvid-gui/locales/zh-CN.toml`

Legacy GUI locale files may be inspected only as wording references. Reuse copy where it still fits, but do not treat `wayvid-gui` or `wayvid-ctl` as active workspace dependencies or shell targets.

## Task 1: Create the New `lwe` Tauri + Svelte Shell

**Files:**
- Create: `apps/lwe/package.json`
- Create: `apps/lwe/svelte.config.js`
- Create: `apps/lwe/vite.config.ts`
- Create: `apps/lwe/tsconfig.json`
- Create: `apps/lwe/src/app.d.ts`
- Create: `apps/lwe/src/routes/+layout.svelte`
- Create: `apps/lwe/src/routes/+page.svelte`
- Create: `apps/lwe/src/routes/library/+page.svelte`
- Create: `apps/lwe/src/routes/workshop/+page.svelte`
- Create: `apps/lwe/src/routes/desktop/+page.svelte`
- Create: `apps/lwe/src/routes/settings/+page.svelte`
- Create: `apps/lwe/src-tauri/Cargo.toml`
- Create: `apps/lwe/src-tauri/tauri.conf.json`
- Create: `apps/lwe/src-tauri/src/main.rs`
- Create: `apps/lwe/src-tauri/src/lib.rs`
- Modify: `Cargo.toml`
- Test: `cargo test -p lwe-app-shell -- --nocapture`

- [ ] **Step 1: Write the failing workspace test for the new app shell crate**

Add this Rust test to `apps/lwe/src-tauri/src/lib.rs` before implementing commands:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn app_name_uses_lwe_code_name() {
        assert_eq!(super::APP_CODE_NAME, "lwe");
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell -- --nocapture`
Expected: FAIL because `apps/lwe/src-tauri` does not exist in the workspace yet.

- [ ] **Step 3: Create the minimal Tauri package manifest**

Create `apps/lwe/src-tauri/Cargo.toml` with:

```toml
[package]
name = "lwe-app-shell"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[dependencies]
tauri = { version = "2", features = [] }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
lwe-core = { path = "../../../crates/lwe-core" }
wayvid-library = { path = "../../../crates/wayvid-library" }
open = "5.0"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

Create `apps/lwe/src-tauri/src/lib.rs` with:

```rust
pub const APP_CODE_NAME: &str = "lwe";

pub fn register_commands() {}

#[cfg(test)]
mod tests {
    #[test]
    fn app_name_uses_lwe_code_name() {
        assert_eq!(super::APP_CODE_NAME, "lwe");
    }
}
```

Create `apps/lwe/src-tauri/src/main.rs` with:

```rust
fn main() {
    lwe_app_shell::register_commands();
}
```

Create `apps/lwe/src-tauri/tauri.conf.json` with the minimal package identity:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "LWE",
  "identifier": "dev.lwe.app",
  "build": {
    "frontendDist": "../build"
  },
  "app": {
    "windows": [
      {
        "title": "LWE",
        "width": 1440,
        "height": 920,
        "resizable": true
      }
    ]
  }
}
```

- [ ] **Step 4: Add the new app shell to the workspace**

In the root `Cargo.toml`, add the new member:

```toml
members = [
    "crates/lwe-core",
    "crates/wayvid-engine",
    "crates/wayvid-library",
    "apps/lwe/src-tauri",
]
```

- [ ] **Step 5: Create the minimal Svelte shell files**

Create `apps/lwe/package.json` with:

```json
{
  "name": "lwe-frontend",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "test": "vitest run"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^4.0.0",
    "svelte": "^5.0.0",
    "svelte-check": "^4.0.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "vitest": "^2.0.0"
  }
}
```

Create `apps/lwe/src/routes/+layout.svelte` with:

```svelte
<script lang="ts">
  const pages = [
    { href: '/library', label: 'Library' },
    { href: '/workshop', label: 'Workshop' },
    { href: '/desktop', label: 'Desktop' },
    { href: '/settings', label: 'Settings' }
  ];
</script>

<nav>
  {#each pages as page}
    <a href={page.href}>{page.label}</a>
  {/each}
</nav>

<slot />
```

Create `apps/lwe/src/routes/+page.svelte` with:

```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  void goto('/library');
</script>
```

Create one placeholder page per route with a heading only, for example `apps/lwe/src/routes/workshop/+page.svelte`:

```svelte
<h1>Workshop</h1>
```

Repeat the same minimal pattern for `library`, `desktop`, and `settings`.

- [ ] **Step 6: Run tests and commit**

Run:

```bash
cargo test -p lwe-app-shell -- --nocapture
```

Expected: PASS

Then:

```bash
git add Cargo.toml apps/lwe
git commit -m "feat: add lwe tauri and svelte shell"
```

## Task 2: Build Rust-Owned Workshop Catalog and Cover Policy

**Files:**
- Create: `crates/wayvid-library/src/workshop_catalog.rs`
- Modify: `crates/wayvid-library/src/lib.rs`
- Modify: `crates/wayvid-library/src/workshop.rs`
- Test: `cargo test -p wayvid-library workshop_catalog::tests -- --nocapture`

- [ ] **Step 1: Write the failing catalog and cover-policy tests**

Create `crates/wayvid-library/src/workshop_catalog.rs` with this test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn bundled_cover_is_used_when_present() {
        let entry = WorkshopCatalogEntry {
            workshop_id: 101,
            title: "Forest Scene".to_string(),
            project_type: WorkshopProjectType::Scene,
            project_dir: PathBuf::from("/tmp/431960/101"),
            cover_path: Some(PathBuf::from("/tmp/431960/101/preview.jpg")),
            sync_state: WorkshopSyncState::Synced,
            supported_first_release: true,
            library_item_id: Some("forest-101".to_string()),
        };

        assert!(entry.has_cover());
    }

    #[test]
    fn unsupported_web_item_has_no_cover_requirement() {
        let entry = WorkshopCatalogEntry {
            workshop_id: 202,
            title: "Interactive Web".to_string(),
            project_type: WorkshopProjectType::Web,
            project_dir: PathBuf::from("/tmp/431960/202"),
            cover_path: None,
            sync_state: WorkshopSyncState::UnsupportedType,
            supported_first_release: false,
            library_item_id: None,
        };

        assert!(!entry.has_cover());
        assert!(!entry.supported_first_release);
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p wayvid-library workshop_catalog::tests -- --nocapture`
Expected: FAIL with missing `WorkshopCatalogEntry`, `WorkshopProjectType`, or `WorkshopSyncState`.

- [ ] **Step 3: Implement the reset-era Workshop catalog model**

Create `crates/wayvid-library/src/workshop_catalog.rs` with:

```rust
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkshopProjectType {
    Video,
    Scene,
    Web,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkshopSyncState {
    Synced,
    MissingProjectFile,
    MissingPrimaryAsset,
    UnsupportedType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopCatalogEntry {
    pub workshop_id: u64,
    pub title: String,
    pub project_type: WorkshopProjectType,
    pub project_dir: PathBuf,
    pub cover_path: Option<PathBuf>,
    pub sync_state: WorkshopSyncState,
    pub supported_first_release: bool,
    pub library_item_id: Option<String>,
}

impl WorkshopCatalogEntry {
    pub fn has_cover(&self) -> bool {
        self.cover_path.is_some()
    }
}
```

- [ ] **Step 4: Re-export and bridge from existing Workshop parsing**

In `crates/wayvid-library/src/lib.rs`, export:

```rust
pub mod workshop_catalog;

pub use workshop_catalog::{
    WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState,
};
```

In `crates/wayvid-library/src/workshop.rs`, add helpers with this shape:

```rust
use crate::workshop_catalog::{
    WorkshopCatalogEntry, WorkshopProjectType, WorkshopSyncState,
};

impl WeProject {
    pub fn project_type_enum(&self) -> WorkshopProjectType {
        match self.project_type.to_lowercase().as_str() {
            "video" => WorkshopProjectType::Video,
            "scene" => WorkshopProjectType::Scene,
            "web" => WorkshopProjectType::Web,
            _ => WorkshopProjectType::Other,
        }
    }

    pub fn cover_image(&self, project_dir: &std::path::Path) -> Option<std::path::PathBuf> {
        self.preview
            .as_ref()
            .map(|preview| project_dir.join(preview))
            .filter(|path| path.exists())
    }
}

impl WorkshopScanner {
    pub fn scan_catalog(&mut self) -> Result<Vec<WorkshopCatalogEntry>> {
        let workshop_paths = self.steam.workshop_content_path(WALLPAPER_ENGINE_APP_ID);
        let mut entries = Vec::new();

        for workshop_path in workshop_paths {
            for item in std::fs::read_dir(&workshop_path)? {
                let item = item?;
                if !item.file_type()?.is_dir() {
                    continue;
                }

                let workshop_id = match item.file_name().to_string_lossy().parse::<u64>() {
                    Ok(id) => id,
                    Err(_) => continue,
                };

                let project_dir = item.path();
                let project_file = project_dir.join("project.json");

                if !project_file.exists() {
                    entries.push(WorkshopCatalogEntry {
                        workshop_id,
                        title: format!("Workshop #{workshop_id}"),
                        project_type: WorkshopProjectType::Other,
                        project_dir,
                        cover_path: None,
                        sync_state: WorkshopSyncState::MissingProjectFile,
                        supported_first_release: false,
                        library_item_id: None,
                    });
                    continue;
                }

                let project = WeProject::load(&project_dir)?;
                let project_type = project.project_type_enum();
                let parsed_item = if matches!(project_type, WorkshopProjectType::Video | WorkshopProjectType::Scene) {
                    self.parse_workshop_item_const(&project_dir, workshop_id)?
                } else {
                    None
                };

                let sync_state = match (project_type, parsed_item.is_some()) {
                    (WorkshopProjectType::Web, _) | (WorkshopProjectType::Other, _) => WorkshopSyncState::UnsupportedType,
                    (_, true) => WorkshopSyncState::Synced,
                    _ => WorkshopSyncState::MissingPrimaryAsset,
                };

                entries.push(WorkshopCatalogEntry {
                    workshop_id,
                    title: project.title.clone().unwrap_or_else(|| format!("Workshop #{workshop_id}")),
                    project_type,
                    project_dir: project_dir.clone(),
                    cover_path: project.cover_image(&project_dir),
                    sync_state,
                    supported_first_release: matches!(project_type, WorkshopProjectType::Video | WorkshopProjectType::Scene)
                        && parsed_item.is_some(),
                    library_item_id: parsed_item.map(|item| item.id),
                });
            }
        }

        entries.sort_by_key(|entry| entry.workshop_id);
        Ok(entries)
    }
}
```

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p wayvid-library workshop_catalog::tests -- --nocapture`
Expected: PASS

Then:

```bash
git add crates/wayvid-library/src/workshop_catalog.rs crates/wayvid-library/src/lib.rs crates/wayvid-library/src/workshop.rs
git commit -m "feat: add lwe workshop catalog model"
```

## Task 3: Define Tauri Snapshot, Detail, and ActionOutcome Models

**Files:**
- Create: `apps/lwe/src-tauri/src/models.rs`
- Create: `apps/lwe/src-tauri/src/action_outcome.rs`
- Create: `apps/lwe/src/lib/types.ts`
- Test: `cargo test -p lwe-app-shell models::tests -- --nocapture`

- [ ] **Step 1: Write the failing model-shape tests**

Add this Rust test module to `apps/lwe/src-tauri/src/models.rs` first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workshop_item_summary_uses_cover_or_placeholder_shape() {
        let item = WorkshopItemSummary {
            id: 42,
            title: "Forest Scene".to_string(),
            item_type: "scene".to_string(),
            cover_path: None,
            sync_status: "synced".to_string(),
            compatibility_badge: "Fully Supported".to_string(),
        };

        assert_eq!(item.item_type, "scene");
        assert!(item.cover_path.is_none());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell models::tests -- --nocapture`
Expected: FAIL with missing `WorkshopItemSummary` or missing `models` module.

- [ ] **Step 3: Implement the Rust snapshot and detail models**

Create `apps/lwe/src-tauri/src/models.rs` with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppShellSnapshot {
    pub app_name: String,
    pub code_name: String,
    pub steam_available: bool,
    pub library_count: usize,
    pub workshop_synced_count: usize,
    pub monitor_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopItemSummary {
    pub id: u64,
    pub title: String,
    pub item_type: String,
    pub cover_path: Option<String>,
    pub sync_status: String,
    pub compatibility_badge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopPageSnapshot {
    pub items: Vec<WorkshopItemSummary>,
    pub selected_item_id: Option<u64>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopItemDetail {
    pub id: u64,
    pub title: String,
    pub item_type: String,
    pub cover_path: Option<String>,
    pub sync_status: String,
    pub compatibility_badge: String,
    pub compatibility_note: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemSummary {
    pub id: String,
    pub title: String,
    pub item_type: String,
    pub cover_path: Option<String>,
    pub source: String,
    pub favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPageSnapshot {
    pub items: Vec<LibraryItemSummary>,
    pub selected_item_id: Option<String>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemDetail {
    pub id: String,
    pub title: String,
    pub item_type: String,
    pub cover_path: Option<String>,
    pub source: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopMonitorSummary {
    pub monitor_id: String,
    pub display_name: String,
    pub resolution: String,
    pub current_wallpaper_title: Option<String>,
    pub current_cover_path: Option<String>,
    pub runtime_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopPageSnapshot {
    pub monitors: Vec<DesktopMonitorSummary>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPageSnapshot {
    pub language: String,
    pub theme: String,
    pub steam_required: bool,
    pub stale: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workshop_item_summary_uses_cover_or_placeholder_shape() {
        let item = WorkshopItemSummary {
            id: 42,
            title: "Forest Scene".to_string(),
            item_type: "scene".to_string(),
            cover_path: None,
            sync_status: "synced".to_string(),
            compatibility_badge: "Fully Supported".to_string(),
        };

        assert_eq!(item.item_type, "scene");
        assert!(item.cover_path.is_none());
    }
}
```

- [ ] **Step 4: Implement `ActionOutcome<T>` and the matching frontend types**

Create `apps/lwe/src-tauri/src/action_outcome.rs` with:

```rust
use serde::{Deserialize, Serialize};

use crate::models::AppShellSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidatedPage {
    Library,
    Workshop,
    Desktop,
    Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppShellPatch {
    pub workshop_synced_count: Option<usize>,
    pub library_count: Option<usize>,
    pub monitor_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutcome<T> {
    pub ok: bool,
    pub message: Option<String>,
    pub shell_patch: Option<AppShellPatch>,
    pub current_update: Option<T>,
    pub invalidations: Vec<InvalidatedPage>,
}

impl<T> ActionOutcome<T> {
    pub fn success(current_update: Option<T>) -> Self {
        Self {
            ok: true,
            message: None,
            shell_patch: None,
            current_update,
            invalidations: Vec::new(),
        }
    }
}
```

Create `apps/lwe/src/lib/types.ts` with matching frontend interfaces:

```ts
export type InvalidatedPage = 'library' | 'workshop' | 'desktop' | 'settings';

export interface AppShellSnapshot {
  appName: string;
  codeName: string;
  steamAvailable: boolean;
  libraryCount: number;
  workshopSyncedCount: number;
  monitorCount: number;
}

export interface WorkshopItemSummary {
  id: number;
  title: string;
  itemType: string;
  coverPath: string | null;
  syncStatus: string;
  compatibilityBadge: string;
}

export interface WorkshopPageSnapshot {
  items: WorkshopItemSummary[];
  selectedItemId: number | null;
  stale: boolean;
}

export interface WorkshopItemDetail {
  id: number;
  title: string;
  itemType: string;
  coverPath: string | null;
  syncStatus: string;
  compatibilityBadge: string;
  compatibilityNote: string | null;
  tags: string[];
  description: string | null;
}

export interface LibraryItemSummary {
  id: string;
  title: string;
  itemType: string;
  coverPath: string | null;
  source: string;
  favorite: boolean;
}

export interface LibraryPageSnapshot {
  items: LibraryItemSummary[];
  selectedItemId: string | null;
  stale: boolean;
}

export interface LibraryItemDetail {
  id: string;
  title: string;
  itemType: string;
  coverPath: string | null;
  source: string;
  description: string | null;
  tags: string[];
}

export interface DesktopMonitorSummary {
  monitorId: string;
  displayName: string;
  resolution: string;
  currentWallpaperTitle: string | null;
  currentCoverPath: string | null;
  runtimeStatus: string;
}

export interface DesktopPageSnapshot {
  monitors: DesktopMonitorSummary[];
  stale: boolean;
}

export interface SettingsPageSnapshot {
  language: string;
  theme: string;
  steamRequired: boolean;
  stale: boolean;
}

export interface AppShellPatch {
  workshopSyncedCount?: number;
  libraryCount?: number;
  monitorCount?: number;
}

export interface ActionOutcome<T> {
  ok: boolean;
  message: string | null;
  shellPatch: AppShellPatch | null;
  currentUpdate: T | null;
  invalidations: InvalidatedPage[];
}
```

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell models::tests -- --nocapture`
Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/models.rs apps/lwe/src-tauri/src/action_outcome.rs apps/lwe/src/lib/types.ts
git commit -m "feat: add lwe snapshot and action outcome models"
```

## Task 4: Implement Tauri Workshop and Library Commands

**Files:**
- Create: `apps/lwe/src-tauri/src/app_shell.rs`
- Create: `apps/lwe/src-tauri/src/workshop.rs`
- Create: `apps/lwe/src-tauri/src/library.rs`
- Create: `apps/lwe/src-tauri/src/desktop.rs`
- Create: `apps/lwe/src-tauri/src/settings.rs`
- Modify: `apps/lwe/src-tauri/src/lib.rs`
- Test: `cargo test -p lwe-app-shell workshop::tests -- --nocapture`

- [ ] **Step 1: Write the failing Workshop command tests**

Add this test module to `apps/lwe/src-tauri/src/workshop.rs` first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_url_uses_official_workshop_page() {
        assert_eq!(
            workshop_item_url(12345),
            "https://steamcommunity.com/sharedfiles/filedetails/?id=12345"
        );
    }

    #[test]
    fn steam_openurl_wraps_official_workshop_page() {
        assert_eq!(
            steam_openurl(12345),
            "steam://openurl/https://steamcommunity.com/sharedfiles/filedetails/?id=12345"
        );
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lwe-app-shell workshop::tests -- --nocapture`
Expected: FAIL with missing `workshop` module or missing URL helpers.

- [ ] **Step 3: Implement app-shell and Workshop query/action commands**

Create `apps/lwe/src-tauri/src/app_shell.rs` with:

```rust
use crate::models::AppShellSnapshot;

pub fn load_app_shell() -> AppShellSnapshot {
    AppShellSnapshot {
        app_name: "LWE".to_string(),
        code_name: "lwe".to_string(),
        steam_available: wayvid_library::SteamLibrary::try_discover().is_some(),
        library_count: 0,
        workshop_synced_count: 0,
        monitor_count: 0,
    }
}
```

Create `apps/lwe/src-tauri/src/workshop.rs` with these core pieces:

```rust
use crate::action_outcome::{ActionOutcome, AppShellPatch, InvalidatedPage};
use crate::models::{WorkshopItemDetail, WorkshopItemSummary, WorkshopPageSnapshot};
use wayvid_library::{WorkshopCatalogEntry, WorkshopProjectType, WorkshopScanner};

pub fn workshop_item_url(workshop_id: u64) -> String {
    format!("https://steamcommunity.com/sharedfiles/filedetails/?id={workshop_id}")
}

pub fn steam_openurl(workshop_id: u64) -> String {
    format!("steam://openurl/{}", workshop_item_url(workshop_id))
}

fn sync_label(entry: &WorkshopCatalogEntry) -> String {
    match entry.sync_state {
        wayvid_library::WorkshopSyncState::Synced => "synced".to_string(),
        wayvid_library::WorkshopSyncState::MissingProjectFile => "missing_project".to_string(),
        wayvid_library::WorkshopSyncState::MissingPrimaryAsset => "missing_asset".to_string(),
        wayvid_library::WorkshopSyncState::UnsupportedType => "unsupported_type".to_string(),
    }
}

fn compatibility_label(entry: &WorkshopCatalogEntry) -> String {
    if entry.supported_first_release {
        "Fully Supported".to_string()
    } else {
        match entry.project_type {
            WorkshopProjectType::Web => "Unsupported".to_string(),
            _ => "Partially Supported".to_string(),
        }
    }
}

fn to_summary(entry: WorkshopCatalogEntry) -> WorkshopItemSummary {
    WorkshopItemSummary {
        id: entry.workshop_id,
        title: entry.title,
        item_type: match entry.project_type {
            WorkshopProjectType::Video => "video".to_string(),
            WorkshopProjectType::Scene => "scene".to_string(),
            WorkshopProjectType::Web => "web".to_string(),
            WorkshopProjectType::Other => "other".to_string(),
        },
        cover_path: entry.cover_path.map(|path| path.to_string_lossy().to_string()),
        sync_status: sync_label(&entry),
        compatibility_badge: compatibility_label(&entry),
    }
}

pub fn load_workshop_page() -> Result<WorkshopPageSnapshot, String> {
    let mut scanner = WorkshopScanner::discover().map_err(|e| e.to_string())?;
    let items = scanner
        .scan_catalog()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(to_summary)
        .collect();

    Ok(WorkshopPageSnapshot {
        items,
        selected_item_id: None,
        stale: false,
    })
}

pub fn load_workshop_item_detail(workshop_id: u64) -> Result<WorkshopItemDetail, String> {
    let mut scanner = WorkshopScanner::discover().map_err(|e| e.to_string())?;
    let entry = scanner
        .scan_catalog()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|entry| entry.workshop_id == workshop_id)
        .ok_or_else(|| format!("Workshop item {workshop_id} not found"))?;

    Ok(WorkshopItemDetail {
        id: entry.workshop_id,
        title: entry.title,
        item_type: match entry.project_type {
            WorkshopProjectType::Video => "video".to_string(),
            WorkshopProjectType::Scene => "scene".to_string(),
            WorkshopProjectType::Web => "web".to_string(),
            WorkshopProjectType::Other => "other".to_string(),
        },
        cover_path: entry.cover_path.map(|path| path.to_string_lossy().to_string()),
        sync_status: sync_label(&entry),
        compatibility_badge: compatibility_label(&entry),
        compatibility_note: Some("Use bundled cover when present; otherwise render a placeholder.".to_string()),
        tags: Vec::new(),
        description: None,
    })
}

pub fn refresh_workshop_catalog() -> Result<ActionOutcome<WorkshopPageSnapshot>, String> {
    let page = load_workshop_page()?;
    let synced_count = page.items.iter().filter(|item| item.sync_status == "synced").count();

    Ok(ActionOutcome {
        ok: true,
        message: Some("Workshop catalog refreshed".to_string()),
        shell_patch: Some(AppShellPatch {
            workshop_synced_count: Some(synced_count),
            library_count: None,
            monitor_count: None,
        }),
        current_update: Some(page),
        invalidations: vec![InvalidatedPage::Library],
    })
}

pub fn open_workshop_in_steam(workshop_id: u64) -> Result<ActionOutcome<()>, String> {
    open::that_detached(steam_openurl(workshop_id)).map_err(|e| e.to_string())?;

    Ok(ActionOutcome {
        ok: true,
        message: Some("Opened item in Steam".to_string()),
        shell_patch: None,
        current_update: None,
        invalidations: Vec::new(),
    })
}
```

- [ ] **Step 4: Implement Library projection and register commands**

Create `apps/lwe/src-tauri/src/library.rs` with:

```rust
use crate::models::{LibraryItemDetail, LibraryItemSummary, LibraryPageSnapshot};
use wayvid_library::WorkshopScanner;

pub fn load_library_page() -> Result<LibraryPageSnapshot, String> {
    let mut scanner = WorkshopScanner::discover().map_err(|e| e.to_string())?;
    let items = scanner
        .scan_all()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|item| LibraryItemSummary {
            id: item.id,
            title: item.name,
            item_type: item.wallpaper_type.as_str().to_string(),
            cover_path: item.thumbnail_path.map(|path| path.to_string_lossy().to_string()),
            source: item.source_type.as_str().to_string(),
            favorite: false,
        })
        .collect();

    Ok(LibraryPageSnapshot {
        items,
        selected_item_id: None,
        stale: false,
    })
}

pub fn load_library_item_detail(item_id: String) -> Result<LibraryItemDetail, String> {
    let page = load_library_page()?;
    let item = page
        .items
        .into_iter()
        .find(|item| item.id == item_id)
        .ok_or_else(|| "Library item not found".to_string())?;

    Ok(LibraryItemDetail {
        id: item.id,
        title: item.title,
        item_type: item.item_type,
        cover_path: item.cover_path,
        source: item.source,
        description: None,
        tags: Vec::new(),
    })
}
```

Create placeholder page loaders for `desktop.rs` and `settings.rs` that return empty but valid snapshots matching the models from Task 3.

Then update `apps/lwe/src-tauri/src/lib.rs` to expose modules and register Tauri commands in a single place, using this skeleton:

```rust
pub mod action_outcome;
pub mod app_shell;
pub mod desktop;
pub mod library;
pub mod models;
pub mod settings;
pub mod workshop;

pub const APP_CODE_NAME: &str = "lwe";

pub fn register_commands() {}
```

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p lwe-app-shell workshop::tests -- --nocapture`
Expected: PASS

Then:

```bash
git add apps/lwe/src-tauri/src/app_shell.rs apps/lwe/src-tauri/src/workshop.rs apps/lwe/src-tauri/src/library.rs apps/lwe/src-tauri/src/desktop.rs apps/lwe/src-tauri/src/settings.rs apps/lwe/src-tauri/src/lib.rs
git commit -m "feat: add lwe tauri workshop and library commands"
```

## Task 5: Build the Thin Svelte Pages, Page Cache, and Detail Flow

**Files:**
- Create: `apps/lwe/src/lib/ipc.ts`
- Create: `apps/lwe/src/lib/stores/ui.ts`
- Create: `apps/lwe/src/lib/components/ItemCard.svelte`
- Create: `apps/lwe/src/lib/components/StatusBadge.svelte`
- Create: `apps/lwe/src/lib/components/CoverImage.svelte`
- Create: `apps/lwe/src/lib/components/WorkshopDetailPanel.svelte`
- Create: `apps/lwe/src/lib/components/LibraryDetailPanel.svelte`
- Modify: `apps/lwe/src/routes/workshop/+page.svelte`
- Modify: `apps/lwe/src/routes/library/+page.svelte`
- Modify: `apps/lwe/src/routes/desktop/+page.svelte`
- Modify: `apps/lwe/src/routes/settings/+page.svelte`
- Test: `npm test --prefix apps/lwe`

- [ ] **Step 1: Write the failing UI-store test for stale-page behavior**

Create `apps/lwe/src/lib/stores/ui.ts` with this test first using Vitest:

```ts
import { describe, expect, it } from 'vitest';

describe('ui page cache', () => {
  it('marks a page stale without dropping the cached snapshot', () => {
    const cache = {
      workshop: { snapshot: { items: [], selectedItemId: null, stale: false }, stale: false }
    };

    cache.workshop.stale = true;

    expect(cache.workshop.snapshot).not.toBeNull();
    expect(cache.workshop.stale).toBe(true);
  });
});
```

- [ ] **Step 2: Run the frontend test to verify it fails**

Run: `npm test --prefix apps/lwe`
Expected: FAIL because the Vitest/Svelte shell is not wired yet or the store file is missing.

- [ ] **Step 3: Implement the thin IPC wrappers and UI state cache**

Create `apps/lwe/src/lib/ipc.ts` with typed wrappers like:

```ts
import { invoke } from '@tauri-apps/api/core';
import type {
  ActionOutcome,
  AppShellSnapshot,
  DesktopPageSnapshot,
  LibraryItemDetail,
  LibraryPageSnapshot,
  SettingsPageSnapshot,
  WorkshopItemDetail,
  WorkshopPageSnapshot
} from '$lib/types';

export const loadAppShell = () => invoke<AppShellSnapshot>('load_app_shell');
export const loadLibraryPage = () => invoke<LibraryPageSnapshot>('load_library_page');
export const loadLibraryItemDetail = (id: string) =>
  invoke<LibraryItemDetail>('load_library_item_detail', { itemId: id });
export const loadWorkshopPage = () => invoke<WorkshopPageSnapshot>('load_workshop_page');
export const loadWorkshopItemDetail = (id: number) =>
  invoke<WorkshopItemDetail>('load_workshop_item_detail', { workshopId: id });
export const refreshWorkshopCatalog = () =>
  invoke<ActionOutcome<WorkshopPageSnapshot>>('refresh_workshop_catalog');
export const openWorkshopInSteam = (id: number) =>
  invoke<ActionOutcome<null>>('open_workshop_in_steam', { workshopId: id });
export const loadDesktopPage = () => invoke<DesktopPageSnapshot>('load_desktop_page');
export const loadSettingsPage = () => invoke<SettingsPageSnapshot>('load_settings_page');
```

Create `apps/lwe/src/lib/stores/ui.ts` with a minimal cache model:

```ts
import { writable } from 'svelte/store';
import type {
  LibraryItemDetail,
  LibraryPageSnapshot,
  WorkshopItemDetail,
  WorkshopPageSnapshot
} from '$lib/types';

export type PageKey = 'library' | 'workshop' | 'desktop' | 'settings';

export const currentPage = writable<PageKey>('library');

export const pageCache = writable({
  library: { snapshot: null as LibraryPageSnapshot | null, detail: null as LibraryItemDetail | null, stale: false },
  workshop: { snapshot: null as WorkshopPageSnapshot | null, detail: null as WorkshopItemDetail | null, stale: false },
  desktop: { snapshot: null as unknown, detail: null, stale: false },
  settings: { snapshot: null as unknown, detail: null, stale: false }
});
```

- [ ] **Step 4: Implement the thin pages and cover-or-placeholder components**

Create `apps/lwe/src/lib/components/CoverImage.svelte` with:

```svelte
<script lang="ts">
  export let coverPath: string | null = null;
  export let label = 'cover';
</script>

{#if coverPath}
  <img src={coverPath} alt={label} />
{:else}
  <div data-placeholder="true">No Cover</div>
{/if}
```

Create `apps/lwe/src/lib/components/StatusBadge.svelte` with:

```svelte
<script lang="ts">
  export let label: string;
</script>

<span>{label}</span>
```

Create `apps/lwe/src/lib/components/ItemCard.svelte` with:

```svelte
<script lang="ts">
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';

  export let title: string;
  export let itemType: string;
  export let coverPath: string | null;
  export let primaryBadge: string;
</script>

<article>
  <CoverImage {coverPath} label={title} />
  <h3>{title}</h3>
  <p>{itemType}</p>
  <StatusBadge label={primaryBadge} />
</article>
```

Create minimal detail panels that accept the typed detail payload and render title, cover, badges, description, and a button in the Workshop panel to call `openWorkshopInSteam`.

Update `apps/lwe/src/routes/workshop/+page.svelte` so it:

- loads `WorkshopPageSnapshot` on mount if no cache exists or the page is stale
- renders `ItemCard` for each summary
- fetches `WorkshopItemDetail` when the selected item changes
- calls `refreshWorkshopCatalog()` for the refresh button
- marks `library` stale when the action outcome invalidates it

Update `apps/lwe/src/routes/library/+page.svelte` so it:

- loads `LibraryPageSnapshot` on mount if needed
- fetches `LibraryItemDetail` on selection
- renders `ItemCard` and `LibraryDetailPanel`

Update the `desktop` and `settings` pages to load and render their snapshots with simple placeholder sections only; keep them thin and truthful.

- [ ] **Step 5: Run tests and commit**

Run: `npm test --prefix apps/lwe`
Expected: PASS

Then:

```bash
git add apps/lwe/src/lib apps/lwe/src/routes
git commit -m "feat: add thin lwe workshop and library frontend"
```

## Task 6: Align Reset-Era Naming and Planning Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/product/roadmap.md`
- Test: `python3` assertions over those files

- [ ] **Step 1: Add the LWE/lwe naming note to `README.md`**

Append this short section after `Product Direction`:

```md
## Naming Direction

- Product name: `LWE`
- Code name and file-path prefix: `lwe`

The old `wayvid` name remains only where legacy crates or migration-candidate assets have not been renamed yet.
```

- [ ] **Step 2: Update `docs/product/roadmap.md` to mention the Tauri + Svelte shell**

Adjust the Workshop planning track so it reads:

```md
- `workshop-browsing-and-acquisition`: build the first `LWE` Workshop loop in the new `lwe` Tauri + Svelte shell using Rust-owned page snapshots, detail payloads, and official Steam handoff actions
```

- [ ] **Step 3: Verify the reset-era naming docs are consistent**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
readme = Path('README.md').read_text()
roadmap = Path('docs/product/roadmap.md').read_text()
assert 'LWE' in readme
assert 'lwe' in readme
assert 'Tauri + Svelte' in roadmap
print('ok')
PY
```

Expected: prints `ok`.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/product/roadmap.md
git commit -m "docs: align workshop track with lwe shell direction"
```

## Self-Review Checklist

- Spec coverage:
  - in-app Workshop browsing surface → Tasks 1, 3, 4, 5
  - item detail in-app → Tasks 3, 4, 5
  - official Steam handoff → Task 4
  - synchronized import into Library → Tasks 2 and 4
  - content-type visibility for `video`, `scene`, `web` → Tasks 2, 3, 4
  - Chinese/English-first product surfaces are not fully expanded here; this plan intentionally establishes the new shell and should be followed by a localized copy pass if the shell ships beyond placeholder text.
- Placeholder scan: no `TODO`, `TBD`, or deferred pseudo-steps appear in the tasks.
- Type consistency:
  - `WorkshopCatalogEntry`, `WorkshopPageSnapshot`, `WorkshopItemDetail`, `LibraryPageSnapshot`, `LibraryItemDetail`, and `ActionOutcome<T>` are introduced before later tasks depend on them.

## Expected Output of This Plan

When this plan is complete, the repository will have:

- a new `LWE` / `lwe` Tauri + Svelte shell checked into the repo
- a Rust-owned Workshop page snapshot and detail model
- official Steam handoff actions rather than direct distribution replacement
- a cover-or-placeholder rendering policy instead of a generated thumbnail system
- synchronized Workshop items projected into the Library page
- a thin frontend cache model based on page snapshots, detail payloads, and stale flags

## Follow-on Plans After This One

The next plans after this file should cover:

1. compatibility reporting and explanation depth for imported Workshop content
2. `video` and `scene` runtime integration under the new `lwe` shell
3. Rust-side application-core layering aligned with the thin frontend contract
