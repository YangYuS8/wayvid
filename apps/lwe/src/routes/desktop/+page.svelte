<script lang="ts">
  import { onMount } from 'svelte';
  import DesktopMonitorCard from '$lib/components/DesktopMonitorCard.svelte';
  import PageHeader from '$lib/layout/PageHeader.svelte';
  import { Card } from '$lib/ui/card';
  import * as Select from '$lib/ui/select';
  import { loadDesktopPage } from '$lib/ipc';
  import { needsPageLoad, pageCache, setCurrentPage, setDesktopSnapshot } from '$lib/stores/ui';
  import { resolveDesktopPageState } from './page-state';

  type MonitorFilter = 'all' | 'active' | 'missing';

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : 'Unable to load the Desktop snapshot.';

  let loading = false;
  let pageError: string | null = null;
  let monitorFilter: MonitorFilter = 'all';

  $: snapshot = $pageCache.desktop.snapshot;
  $: pageState = snapshot ? resolveDesktopPageState(snapshot) : null;
  $: visibleMonitors = monitorFilter === 'missing' ? [] : snapshot?.monitors ?? [];
  $: visibleMissingMonitorRestores = monitorFilter === 'active' ? [] : snapshot?.missingMonitorRestores ?? [];
  $: filterEmptyMessage =
    monitorFilter === 'active'
      ? 'No active monitors are available in the current snapshot.'
      : monitorFilter === 'missing'
        ? 'No missing monitor restores are recorded in the current snapshot.'
        : null;

  const ensurePage = async () => {
    if (!needsPageLoad('desktop')) {
      return;
    }

    loading = true;
    pageError = null;

    try {
      setDesktopSnapshot(await loadDesktopPage());
    } catch (error) {
      pageError = readError(error);
    } finally {
      loading = false;
    }
  };

  onMount(() => {
    setCurrentPage('desktop');
    void ensurePage();
  });
</script>

<svelte:head>
  <title>Desktop</title>
</svelte:head>

<section class="page">
  <PageHeader
    eyebrow="Desktop"
    title="Monitor shell"
    subtitle="Render the current desktop snapshot without inventing runtime behavior in the frontend."
  >
    {#snippet actions()}
      <label class="monitor-filter">
        <span>View</span>
        <Select.Root type="single" name="monitorFilter" bind:value={monitorFilter}>
          <Select.Trigger aria-label="Monitor view filter" class="min-w-[11rem]">
            {monitorFilter === 'all'
              ? 'All outputs'
              : monitorFilter === 'active'
                ? 'Current monitors'
                : 'Missing restores'}
          </Select.Trigger>

          <Select.Content>
            <Select.Item value="all" label="All outputs">All outputs</Select.Item>
            <Select.Item value="active" label="Current monitors">Current monitors</Select.Item>
            <Select.Item value="missing" label="Missing restores">Missing restores</Select.Item>
          </Select.Content>
        </Select.Root>
      </label>
    {/snippet}
  </PageHeader>

  {#if pageError}
    <p class="message error" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !$pageCache.desktop.snapshot}
    <p role="status" aria-live="polite">Loading Desktop snapshot…</p>
  {:else if snapshot}
    <div class="page-body">
      <Card class="summary-panel border-slate-200/80 bg-white/95 p-5 shadow-[0_24px_60px_rgba(15,23,42,0.08)]">
        <div class="summary-grid">
          <div class="summary-card">
            <p class="summary-label">Monitors discovered</p>
            <p class="summary-value">{snapshot.monitors.length}</p>
          </div>

          <div class="summary-card">
            <p class="summary-label">Monitor discovery</p>
            <p class="summary-copy">{pageState?.monitorAvailabilityLabel ?? 'no'}</p>
          </div>

          <div class="summary-card">
            <p class="summary-label">Assignment persistence</p>
            <p class="summary-copy">{pageState?.assignmentAvailabilityLabel ?? 'no'}</p>
          </div>

          <div class="summary-card">
            <p class="summary-label">Snapshot stale</p>
            <p class="summary-copy">{snapshot.stale ? 'yes' : 'no'}</p>
          </div>
        </div>

        {#if pageState?.issueMessages.length}
          <div class="issues" aria-live="polite">
            {#each pageState.issueMessages as issue}
              <p class="message warning">{issue}</p>
            {/each}
          </div>
        {/if}

        {#if pageState?.emptyMessage}
          <p class="summary-copy">{pageState.emptyMessage}</p>
        {/if}
      </Card>

      {#if visibleMonitors.length > 0}
        <section class="section-block">
          <div class="section-heading">
            <p class="section-kicker">Active outputs</p>
            <h2>Current monitors</h2>
          </div>

          <div class="monitor-grid">
            {#each visibleMonitors as monitor}
              <DesktopMonitorCard
                displayName={monitor.displayName}
                monitorId={monitor.monitorId}
                resolution={monitor.resolution}
                currentItemLabel={monitor.currentWallpaperTitle ?? monitor.currentItemId ?? 'No saved assignment'}
                currentCoverPath={monitor.currentCoverPath}
                runtimeStatus={monitor.runtimeStatus}
                restoreState={monitor.restoreState ?? null}
                restoreIssue={monitor.restoreIssue ?? null}
              />
            {/each}
          </div>
        </section>
      {:else if monitorFilter === 'active'}
        <p class="summary-copy" role="status" aria-live="polite">{filterEmptyMessage}</p>
      {/if}

      {#if visibleMissingMonitorRestores.length > 0}
        <section class="section-block">
          <h2>Missing monitor restores</h2>

          <div class="monitor-grid">
            {#each visibleMissingMonitorRestores as restore}
              <DesktopMonitorCard
                displayName={restore.monitorId}
                monitorId={restore.monitorId}
                resolution={null}
                currentItemLabel={restore.currentWallpaperTitle ?? restore.currentItemId}
                currentCoverPath={null}
                restoreState={restore.restoreState}
                restoreIssue={restore.restoreIssue ?? null}
                missing={true}
              />
            {/each}
          </div>
        </section>
      {:else if monitorFilter === 'missing'}
        <p class="summary-copy" role="status" aria-live="polite">{filterEmptyMessage}</p>
      {/if}

      <p class="footnote">The runtime control surface stays deferred until a later task exposes real commands.</p>
    </div>
  {/if}
</section>

<style>
  .page,
  .page-body,
  .summary-grid,
  .summary-card,
  .issues,
  .monitor-filter,
  .section-block,
  .section-heading,
  .monitor-grid {
    display: grid;
    gap: 1rem;
  }

  p,
  h2 {
    margin: 0;
  }

  .summary-grid {
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  }

  .summary-card {
    align-content: start;
    padding: 1rem;
    border-radius: 18px;
    border: 1px solid rgba(148, 163, 184, 0.24);
    background: rgba(248, 250, 252, 0.72);
  }

  .summary-label,
  .section-kicker {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: #526272;
  }

  .summary-value {
    font-size: clamp(1.75rem, 3vw, 2.4rem);
    font-weight: 700;
    color: #0f172a;
  }

  .summary-copy,
  .footnote {
    color: #526272;
    line-height: 1.55;
  }

  .monitor-filter {
    justify-items: start;
    gap: 0.4rem;
  }

  .monitor-filter span {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: #526272;
  }

  .section-heading {
    gap: 0.35rem;
  }

  .monitor-grid {
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  }

  h2 {
    font-size: 1.15rem;
    color: #0f172a;
  }

  .message.error {
    padding: 0.85rem 1rem;
    border-radius: 14px;
    background: rgba(160, 98, 23, 0.12);
  }

  .message.warning {
    padding: 0.85rem 1rem;
    border-radius: 14px;
    background: rgba(15, 95, 154, 0.12);
  }

  @media (max-width: 700px) {
    .summary-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
