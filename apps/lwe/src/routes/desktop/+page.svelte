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

<section class="grid gap-6">
  <PageHeader
    eyebrow="Desktop"
    title="Monitor shell"
    subtitle="Render the current desktop snapshot without inventing runtime behavior in the frontend."
  >
    {#snippet actions()}
      <label class="grid justify-items-start gap-1.5">
        <span class="lwe-eyebrow">View</span>
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
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !$pageCache.desktop.snapshot}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">Loading Desktop snapshot…</p>
  {:else if snapshot}
    <div class="grid gap-5">
      <Card class="lwe-panel gap-5">
        <div class="grid gap-4 [grid-template-columns:repeat(auto-fit,minmax(180px,1fr))]">
          <div class="lwe-subpanel content-start gap-2.5">
            <p class="lwe-eyebrow">Monitors discovered</p>
            <p class="text-[clamp(1.75rem,3vw,2.4rem)] font-semibold tracking-tight text-slate-950">
              {snapshot.monitors.length}
            </p>
          </div>

          <div class="lwe-subpanel content-start gap-2.5">
            <p class="lwe-eyebrow">Monitor discovery</p>
            <p class="text-sm leading-6 text-slate-600">{pageState?.monitorAvailabilityLabel ?? 'no'}</p>
          </div>

          <div class="lwe-subpanel content-start gap-2.5">
            <p class="lwe-eyebrow">Assignment persistence</p>
            <p class="text-sm leading-6 text-slate-600">{pageState?.assignmentAvailabilityLabel ?? 'no'}</p>
          </div>

          <div class="lwe-subpanel content-start gap-2.5">
            <p class="lwe-eyebrow">Snapshot stale</p>
            <p class="text-sm leading-6 text-slate-600">{snapshot.stale ? 'yes' : 'no'}</p>
          </div>
        </div>

        {#if pageState?.issueMessages.length}
          <div class="grid gap-2.5" aria-live="polite">
            {#each pageState.issueMessages as issue}
              <p class="lwe-info-banner">{issue}</p>
            {/each}
          </div>
        {/if}

        {#if pageState?.emptyMessage}
          <p class="text-sm leading-6 text-slate-600">{pageState.emptyMessage}</p>
        {/if}
      </Card>

      {#if visibleMonitors.length > 0}
        <section class="grid gap-4">
          <div class="grid gap-1.5">
            <p class="lwe-eyebrow">Active outputs</p>
            <h2 class="lwe-heading-md">Current monitors</h2>
          </div>

          <div class="grid gap-4 [grid-template-columns:repeat(auto-fit,minmax(220px,1fr))]">
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
        <p class="text-sm leading-6 text-slate-600" role="status" aria-live="polite">{filterEmptyMessage}</p>
      {/if}

      {#if visibleMissingMonitorRestores.length > 0}
        <section class="grid gap-4">
          <h2 class="lwe-heading-md">Missing monitor restores</h2>

          <div class="grid gap-4 [grid-template-columns:repeat(auto-fit,minmax(220px,1fr))]">
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
        <p class="text-sm leading-6 text-slate-600" role="status" aria-live="polite">{filterEmptyMessage}</p>
      {/if}

      <p class="text-sm leading-6 text-slate-500">
        The runtime control surface stays deferred until a later task exposes real commands.
      </p>
    </div>
  {/if}
</section>
