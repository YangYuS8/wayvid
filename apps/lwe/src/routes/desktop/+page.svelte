<script lang="ts">
  import { onMount } from 'svelte';
  import { loadDesktopPage } from '$lib/ipc';
  import { needsPageLoad, pageCache, setCurrentPage, setDesktopSnapshot } from '$lib/stores/ui';
  import { resolveDesktopPageState } from './page-state';

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : 'Unable to load the Desktop snapshot.';

  let loading = false;
  let pageError: string | null = null;

  $: snapshot = $pageCache.desktop.snapshot;
  $: pageState = snapshot ? resolveDesktopPageState(snapshot) : null;

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

<section class="page-shell">
  <header>
    <p class="eyebrow">Desktop</p>
    <h1>Monitor shell</h1>
    <p>Render the current desktop snapshot without inventing runtime behavior in the frontend.</p>
  </header>

  {#if pageError}
    <p class="message error" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !$pageCache.desktop.snapshot}
    <p role="status" aria-live="polite">Loading Desktop snapshot…</p>
  {:else if snapshot}
    <section class="panel">
      <p>Monitors discovered: {snapshot.monitors.length}</p>
      <p>Monitor discovery available: {pageState?.monitorAvailabilityLabel ?? 'no'}</p>
      <p>Assignment persistence available: {pageState?.assignmentAvailabilityLabel ?? 'no'}</p>
      <p>Snapshot stale: {snapshot.stale ? 'yes' : 'no'}</p>

      {#if pageState?.issueMessages.length}
        <div class="issues" aria-live="polite">
          {#each pageState.issueMessages as issue}
            <p class="message warning">{issue}</p>
          {/each}
        </div>
      {/if}

      {#if pageState?.emptyMessage}
        <p>{pageState.emptyMessage}</p>
      {/if}

      <p>The runtime control surface stays deferred until a later task exposes real commands.</p>
    </section>
  {/if}
</section>

<style>
  .page-shell,
  header,
  .panel,
  .issues {
    display: grid;
    gap: 1rem;
  }

  .page-shell {
    padding: 1.5rem;
  }

  .eyebrow,
  h1,
  p {
    margin: 0;
  }

  .eyebrow {
    font-size: 0.8rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: #5a6978;
  }

  .panel {
    border: 1px solid rgba(33, 52, 72, 0.12);
    border-radius: 22px;
    padding: 1.1rem;
    background: rgba(255, 255, 255, 0.92);
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
</style>
