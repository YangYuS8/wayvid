# Workshop Online Search and Basic Filters Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a Steam Workshop online search experience in the Workshop page with persisted query state, live debounced search, and basic age/type filters, while keeping Steam-client subscription behavior unchanged.

**Architecture:** Add a new backend search command path that fetches and normalizes online Workshop entries, classifies age rating, filters by type/rating, and paginates results. Extend frontend Workshop page state to drive query/filter controls, request orchestration, and stale-response safety. Persist query/filter preferences through existing settings persistence so first-run defaults and last-used restoration are deterministic.

**Tech Stack:** Tauri v2 command/service layers (Rust), existing `lwe-library` integration points, Svelte page/store logic, existing IPC wrapper, TypeScript tests, Rust unit tests

---

## File Map

- Create: `src-tauri/src/results/workshop_online.rs`
  - Defines backend request/response/domain types for online query/filter/rating payloads.
- Create: `src-tauri/src/assembly/workshop_online.rs`
  - Maps service result into API models consumed by frontend.
- Modify: `src-tauri/src/models.rs`
  - Add serialized models for online search item summary, filter enums, and search response.
- Modify: `src-tauri/src/services/workshop_service.rs`
  - Add online search implementation, age-rating classification, filtering, pagination.
- Modify: `src-tauri/src/commands/workshop.rs`
  - Add `search_workshop_online` command.
- Modify: `src-tauri/src/lib.rs`
  - Register `search_workshop_online` command.
- Modify: `src-tauri/src/results/settings_persistence.rs`
  - Extend persisted settings with workshop query/filter preferences + defaults.
- Modify: `src-tauri/src/services/settings_persistence_service.rs`
  - Keep round-trip tests up to date for new settings fields.
- Modify: `src/lib/types.ts`
  - Add online workshop request/response/item/filter TypeScript types.
- Modify: `src/lib/ipc.ts`
  - Add `searchWorkshopOnline` IPC binding.
- Modify: `src/lib/i18n.ts`
  - Add Workshop search/filter UI copy keys in EN and zh-CN.
- Modify: `src/lib/stores/ui.ts`
  - Add workshop online-query state cache primitives.
- Modify: `src/routes/workshop/page-state.ts`
  - Add query/filter transition helpers and stale-request token helpers for online search.
- Modify: `src/routes/workshop/+page.svelte`
  - Add search input, age/type filters, load-more control, and online result rendering flow.
- Test: `src/routes/workshop/page-state.test.ts`
  - Add frontend state logic tests for query/filter/reset/debounce token flow.
- Test: `src/routes/workshop/page-render.test.ts`
  - Add UI behavior tests for controls, empty/error/loading states.
- Test: `src-tauri/src/services/workshop_service.rs` (inline tests)
  - Add unit tests for rating mapping/filter/pagination behavior.
- Test: `src-tauri/src/commands/workshop.rs` (inline tests)
  - Add command-level request shape and mapping tests where possible.

## Task 1: Define Backend Online Search Models and Contracts

**Files:**
- Create: `src-tauri/src/results/workshop_online.rs`
- Create: `src-tauri/src/assembly/workshop_online.rs`
- Modify: `src-tauri/src/models.rs`

- [ ] **Step 1: Write failing Rust model test for new online payload serialization**

Add tests in `src-tauri/src/models.rs` asserting JSON shape for:

- `WorkshopOnlineSearchRequest`
- `WorkshopOnlineSearchResponse`
- `WorkshopAgeRating`
- `WorkshopItemTypeFilter`

Run:

```bash
cargo test -p lwe-shell models_serialize_workshop_online -- --nocapture
```

Expected: FAIL because types do not exist yet.

- [ ] **Step 2: Add Rust API models and enums**

In `src-tauri/src/models.rs`, add serializable types:

- `WorkshopAgeRating` (`g`, `pg_13`, `r_18`)
- `WorkshopItemTypeFilter` (`video`, `scene`, `web`, `application`)
- `WorkshopOnlineSearchRequest`
- `WorkshopOnlineItemSummary`
- `WorkshopOnlineSearchResponse`

Use `#[serde(rename_all = "snake_case")]` where enum wire format requires it and `camelCase` for struct fields to match existing API conventions.

- [ ] **Step 3: Add service/result layer domain structs for online search processing**

Create `src-tauri/src/results/workshop_online.rs` containing internal structs used by service/assembly (including `age_rating_reason`).

- [ ] **Step 4: Add assembly mapping for online search response**

Create `src-tauri/src/assembly/workshop_online.rs` with function mapping service result -> `WorkshopOnlineSearchResponse` model.

- [ ] **Step 5: Run targeted tests to verify model/assembly green**

Run:

```bash
cargo test -p lwe-shell models_serialize_workshop_online -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Commit Task 1**

```bash
git add src-tauri/src/models.rs src-tauri/src/results/workshop_online.rs src-tauri/src/assembly/workshop_online.rs
git commit -m "feat: define workshop online search contracts"
```

Expected: commit created.

## Task 2: Implement Backend Service Search, Rating, Filtering, and Pagination

**Files:**
- Modify: `src-tauri/src/services/workshop_service.rs`
- Test: `src-tauri/src/services/workshop_service.rs` (inline tests)

- [ ] **Step 1: Write failing unit tests for rating mapping and filter behavior**

Add tests covering:

- adult marker -> `R-18`
- mature marker -> `PG-13`
- fallback safe -> `G`
- type filter include/exclude behavior
- pagination `has_more` and page slicing

Run:

```bash
cargo test -p lwe-shell workshop_service_online -- --nocapture
```

Expected: FAIL due to missing implementation.

- [ ] **Step 2: Add online search entry-point in service**

In `src-tauri/src/services/workshop_service.rs`, implement:

- `search_online(...) -> Result<...>`
- query validation/normalization
- remote fetch adapter call (through existing/extended library integration point)

- [ ] **Step 3: Implement rating classifier and reason-code emission**

Add pure helper function(s):

- classify metadata into `WorkshopAgeRating`
- return reason string (`adult_tag`, `mature_tag`, `default_safe`, ...)

- [ ] **Step 4: Implement filter pipeline and pagination**

Apply pipeline in deterministic order:

1. query results
2. type filter
3. age filter
4. page slicing + `has_more`

- [ ] **Step 5: Run service tests to verify green**

Run:

```bash
cargo test -p lwe-shell workshop_service_online -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Commit Task 2**

```bash
git add src-tauri/src/services/workshop_service.rs
git commit -m "feat: add workshop online search filtering and pagination"
```

Expected: commit created.

## Task 3: Expose New Tauri Command and Wire Backend Registration

**Files:**
- Modify: `src-tauri/src/commands/workshop.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/commands/workshop.rs`

- [ ] **Step 1: Write failing command test for online search command shape**

Add command-level test that validates request -> service -> response mapping path for basic case.

Run:

```bash
cargo test -p lwe-shell workshop_command_online_search -- --nocapture
```

Expected: FAIL.

- [ ] **Step 2: Implement `search_workshop_online` command**

In `src-tauri/src/commands/workshop.rs`, add new `#[tauri::command]` function accepting search request model and returning search response model.

- [ ] **Step 3: Register command in builder**

In `src-tauri/src/lib.rs`, add command to `generate_handler![]` list.

- [ ] **Step 4: Run command tests**

```bash
cargo test -p lwe-shell workshop_command_online_search -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit Task 3**

```bash
git add src-tauri/src/commands/workshop.rs src-tauri/src/lib.rs
git commit -m "feat: expose workshop online search tauri command"
```

Expected: commit created.

## Task 4: Persist Workshop Query and Filter Preferences

**Files:**
- Modify: `src-tauri/src/results/settings_persistence.rs`
- Modify: `src-tauri/src/services/settings_persistence_service.rs`

- [ ] **Step 1: Write failing persistence tests for new workshop preferences fields**

Add tests asserting defaults and round-trip for:

- `workshop_query`
- `workshop_age_ratings`
- `workshop_item_types`

Run:

```bash
cargo test -p lwe-shell settings_persistence_round_trips_workshop_preferences -- --nocapture
```

Expected: FAIL due to missing fields.

- [ ] **Step 2: Extend persisted settings struct with defaults**

In `src-tauri/src/results/settings_persistence.rs`, add fields with defaults:

- first-run age ratings default to `G + PG-13`
- first-run item types default to all four types
- query default empty string

- [ ] **Step 3: Update persistence service tests and fixtures**

Adjust existing TOML fixture expectations in `settings_persistence_service.rs` tests to include or tolerate new fields where needed.

- [ ] **Step 4: Run persistence tests**

```bash
cargo test -p lwe-shell settings_persistence -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit Task 4**

```bash
git add src-tauri/src/results/settings_persistence.rs src-tauri/src/services/settings_persistence_service.rs
git commit -m "feat: persist workshop online search preferences"
```

Expected: commit created.

## Task 5: Add Frontend Types and IPC Binding

**Files:**
- Modify: `src/lib/types.ts`
- Modify: `src/lib/ipc.ts`

- [ ] **Step 1: Write failing TS test for new IPC contract function**

Add/extend test in `src/lib/ipc.test.ts` verifying `searchWorkshopOnline` invokes `search_workshop_online` with expected payload.

Run:

```bash
pnpm test src/lib/ipc.test.ts
```

Expected: FAIL because function/types missing.

- [ ] **Step 2: Add TypeScript models for online search request/response**

In `src/lib/types.ts`, add matching interfaces/enums for online search payload.

- [ ] **Step 3: Add IPC wrapper method**

In `src/lib/ipc.ts`, add:

- `searchWorkshopOnline(input)` calling `invoke('search_workshop_online', { input })`

- [ ] **Step 4: Run IPC tests**

```bash
pnpm test src/lib/ipc.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit Task 5**

```bash
git add src/lib/types.ts src/lib/ipc.ts src/lib/ipc.test.ts
git commit -m "feat: add workshop online search frontend contracts"
```

Expected: commit created.

## Task 6: Add Workshop Search/Filter UI Copy and State Helpers

**Files:**
- Modify: `src/lib/i18n.ts`
- Modify: `src/routes/workshop/page-state.ts`
- Test: `src/routes/workshop/page-state.test.ts`

- [ ] **Step 1: Write failing tests for workshop page-state query transition logic**

Add tests for:

- query/filter change resets page to 1
- load-more increments page
- stale request token invalidation

Run:

```bash
pnpm test src/routes/workshop/page-state.test.ts
```

Expected: FAIL.

- [ ] **Step 2: Add i18n copy keys for workshop search controls**

In `src/lib/i18n.ts` EN + zh-CN dictionaries add:

- search placeholder/button/loading copy
- filter group labels and options (`G`, `PG-13`, `R-18`, `Video`, `Scene`, `Web`, `Application`)
- empty/error context copy for online results

- [ ] **Step 3: Implement page-state helpers for query/filter orchestration**

In `src/routes/workshop/page-state.ts`, add pure helper functions for:

- initial/default query state
- persistence merge/normalization
- pagination transitions
- request token stale-discard control

- [ ] **Step 4: Run page-state tests**

```bash
pnpm test src/routes/workshop/page-state.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit Task 6**

```bash
git add src/lib/i18n.ts src/routes/workshop/page-state.ts src/routes/workshop/page-state.test.ts
git commit -m "feat: add workshop search filter copy and state helpers"
```

Expected: commit created.

## Task 7: Implement Workshop Page Online Search Experience

**Files:**
- Modify: `src/lib/stores/ui.ts`
- Modify: `src/routes/workshop/+page.svelte`
- Test: `src/routes/workshop/page-render.test.ts`

- [ ] **Step 1: Write failing render tests for workshop controls and behavior**

Add tests for:

- search input and filter controls render
- debounce search trigger behavior (using fake timers if existing setup supports it)
- Enter/button immediate trigger
- load-more control when `hasMore=true`

Run:

```bash
pnpm test src/routes/workshop/page-render.test.ts
```

Expected: FAIL prior to implementation.

- [ ] **Step 2: Extend UI store for workshop online state**

In `src/lib/stores/ui.ts`, add workshop online search sub-state fields:

- query/filter selections
- current page
- hasMore
- totalApprox
- loading/error flags

Ensure existing workshop detail selection logic remains intact.

- [ ] **Step 3: Update Workshop page UI and request orchestration**

In `src/routes/workshop/+page.svelte`:

- add search input + search button
- add age/type multi-select controls
- wire debounce + immediate trigger
- call `searchWorkshopOnline`
- reset page on condition changes
- add load-more behavior
- keep open-in-Steam action unchanged

- [ ] **Step 4: Run render tests**

```bash
pnpm test src/routes/workshop/page-render.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit Task 7**

```bash
git add src/lib/stores/ui.ts src/routes/workshop/+page.svelte src/routes/workshop/page-render.test.ts
git commit -m "feat: add workshop online search and basic filter UI"
```

Expected: commit created.

## Task 8: Integrate Settings Persistence with Workshop Query State Restore

**Files:**
- Modify: `src-tauri/src/services/settings_service.rs`
- Modify: `src-tauri/src/commands/settings.rs`
- Modify: `src/routes/workshop/+page.svelte`

- [ ] **Step 1: Write failing test for first-run default workshop filter state**

Add backend or frontend test (where easier with current architecture) asserting first-run defaults:

- age ratings = `G + PG-13`
- item types = all

Run targeted test command based on file added.

Expected: FAIL.

- [ ] **Step 2: Expose/consume workshop preference fields in settings page data flow**

Wire persisted workshop preferences into structures consumed by workshop page initialization path.

- [ ] **Step 3: Apply restored state on Workshop page mount and trigger initial query**

Ensure page first query uses restored state and not hardcoded defaults.

- [ ] **Step 4: Run focused tests**

Run:

```bash
cargo test -p lwe-shell settings_service -- --nocapture
pnpm test src/routes/workshop/page-state.test.ts src/routes/workshop/page-render.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit Task 8**

```bash
git add src-tauri/src/services/settings_service.rs src-tauri/src/commands/settings.rs src/routes/workshop/+page.svelte
git commit -m "feat: restore workshop search preferences on load"
```

Expected: commit created.

## Task 9: Full Verification

**Files:**
- Verify all files modified by Tasks 1-8

- [ ] **Step 1: Run Rust tests for touched backend modules**

```bash
cargo test -p lwe-shell workshop_service_online workshop_command_online_search settings_persistence -- --nocapture
```

Expected: PASS.

- [ ] **Step 2: Run frontend Workshop tests**

```bash
pnpm test src/routes/workshop/page-state.test.ts src/routes/workshop/page-render.test.ts src/lib/ipc.test.ts
```

Expected: PASS.

- [ ] **Step 3: Run lint/typecheck/build verification used in repo**

```bash
pnpm test
cargo check -p lwe-shell
```

Expected: PASS with no new warnings/errors attributable to this feature.

- [ ] **Step 4: Sanity-check local behavior manually**

Run app and validate:

- query updates list with debounce
- Enter/button triggers immediate query
- filter toggles affect results
- load-more works
- subscription action still opens Steam

- [ ] **Step 5: Final integration commit (only if previous tasks were not committed individually)**

```bash
git add -A
git commit -m "feat: add online Steam Workshop search with basic filtering"
```

Expected: commit created if needed.

## Plan Self-Review

- Spec coverage: online search, age/type filters, persisted last-used state, first-run defaults, debounce + immediate search, pagination, and preserved Steam subscription flow are all mapped to explicit tasks.
- Placeholder scan: no TODO/TBD placeholders remain; each task includes concrete files and commands.
- Type consistency: request/response/filter names are consistent across backend models, IPC, store, and page UI.
