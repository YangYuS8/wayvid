# Workshop Online Search and Basic Filtering Design

## Goal

Upgrade the Workshop page to behave like Wallpaper Engine browsing: users can search Steam Workshop online content and apply basic filters for age rating and wallpaper type, while subscription still happens through the Steam client.

## Scope

In scope:

- Steam Workshop **online** search as primary data source for Workshop browsing
- Search box with debounce and manual trigger support
- Basic filters:
  - Age rating: `G`, `PG-13`, `R-18`
  - Wallpaper type: `video`, `scene`, `web`, `application`
- Persist and restore last-used search/filter state
- Keep existing Steam subscription/open flow (`steam://openurl/...`)

Out of scope:

- Resolution filtering
- Tag filtering
- Replacing Steam-based subscribe behavior with direct API subscription

## Product Decisions Confirmed

- Data source: **A. direct online Steam Workshop search**
- Age filter default behavior: **remember last choice; first use defaults to `G + PG-13`**
- Search trigger: **debounced live search (recommended), plus immediate submit on Enter/button**
- Implementation approach: **Backend-centric search service (recommended Option 1)**

## Architecture

### Existing baseline

Current Workshop page (`src/routes/workshop/+page.svelte`) loads local scanned catalog via `load_workshop_page` and supports detail loading plus open-in-Steam actions.

### New structure

Introduce an online-search path in parallel with current local-scan path:

1. **Backend command layer (`src-tauri/src/commands/workshop.rs`)**
   - Add `search_workshop_online` command
2. **Backend service layer (`src-tauri/src/services/workshop_service.rs`)**
   - Add online query execution, type filtering, age-rating mapping, and pagination
3. **Frontend Workshop page (`src/routes/workshop/+page.svelte`)**
   - Add search/filter controls and online result rendering
4. **Frontend state helpers (`src/routes/workshop/page-state.ts`)**
   - Add query-state normalization and stale-request handling helpers
5. **Persistence integration (existing settings persistence path in `src-tauri`)**
   - Save/restore workshop query and filter preferences

The local catalog refresh path remains for current compatibility and incremental rollout, but online results become the primary browsing feed in the Workshop view.

## Data Contract

### New command request model

- `query: String`
- `age_ratings: Vec<AgeRatingFilter>`
- `item_types: Vec<ItemTypeFilter>`
- `page: u32`
- `page_size: u32`

### New command response model

- `items: Vec<WorkshopOnlineItemSummary>`
- `page: u32`
- `has_more: bool`
- `total_approx: Option<u32>`

Each returned item includes:

- current summary fields used by cards (`id`, `title`, `item_type`, `cover_path`, compatibility summary)
- `age_rating` (`g`, `pg_13`, `r_18`)
- `age_rating_reason` (`adult_tag`, `mature_tag`, `default_safe`, etc.)
- source marker (`online`)

## Filtering Semantics

### Type filter

Direct mapping with multi-select:

- `video`
- `scene`
- `web`
- `application`

Default: all selected.

### Age rating filter

Steam content metadata is mapped to app-level tiers:

- `R-18`: clear adult/NSFW markers
- `PG-13`: mature but not explicit-adult markers
- `G`: all remaining content

The service returns both mapped class and reason code for observability and future tuning.

Default behavior:

- first run: `G + PG-13`
- subsequent runs: restore last saved user selection

## UX and Interaction Design

## Workshop top controls

- Search input with 400ms debounce
- Enter key and explicit Search button trigger immediate query
- Filter groups:
  - age rating checkboxes/toggles (`G`, `PG-13`, `R-18`)
  - item type checkboxes/toggles (`Video`, `Scene`, `Web`, `Application`)

## Results flow

- Condition change (`query`, filters) resets to page 1
- Initial page loads with restored/sensible defaults
- Pagination uses explicit **Load more** button (not infinite scroll) for predictable behavior
- Existing detail panel remains usable with selected item

## Subscription behavior

- Keep current Steam open/subscribe flow unchanged via existing command path

## State Persistence

Persist and restore:

- `query`
- `selected_age_ratings`
- `selected_item_types`

Storage should reuse existing app settings persistence mechanism instead of introducing a separate file.

## Error Handling

- Online fetch/network failure: show explicit request error in Workshop page
- Parse/mapping failure for individual item: skip invalid entries where possible and keep list usable
- Empty result set: show dedicated empty state for current query/filter combination
- Request race safety: stale responses are discarded based on request token sequence

No automatic fallback to local-only search for this feature phase, because online search is the chosen primary mode.

## Testing Plan

### Backend tests

- Age-rating mapping rules (`R-18`, `PG-13`, `G`) with reason codes
- Type filter behavior for single/multi-selection
- Pagination boundaries and `has_more` correctness
- Empty search responses and invalid item resilience

### Frontend tests

- Debounced query execution and immediate Enter/button search
- Filter change resets pagination
- Persistence restore on page re-entry
- Stale request discard behavior

### End-to-end acceptance checks

- First run defaults to `G + PG-13`
- Last used filter selections restored on next entry
- Disabling `R-18` suppresses adult-classified entries
- Search and filter updates reflect without page restart
- Open-in-Steam action still works exactly as current flow

## Rollout Notes

- Keep local Workshop scan code path available during transition
- Add internal logging around age-rating reason distribution to refine mapping quality over time

## Risks and Mitigations

- **Steam metadata variability** may produce age-rating misclassification
  - Mitigation: explicit reason codes + rules table refinement
- **Remote API changes** may affect parsing
  - Mitigation: isolate parser logic in service layer and keep robust fallback for partial results
- **Request burst from live search**
  - Mitigation: debounce, request cancellation/ignore stale responses, bounded page size
