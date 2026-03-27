<script lang="ts">
  import { onMount } from 'svelte';
  import { loadDesktopPage } from '$lib/ipc';
  import { needsPageLoad, pageCache, setCurrentPage, setDesktopSnapshot } from '$lib/stores/ui';

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : 'Unable to load the Desktop snapshot.';

  let loading = false;
  let pageError: string | null = null;

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
  {:else if $pageCache.desktop.snapshot}
    <section class="panel">
      <p>Monitors discovered: {$pageCache.desktop.snapshot.monitors.length}</p>
      <p>Snapshot stale: {$pageCache.desktop.snapshot.stale ? 'yes' : 'no'}</p>
      <p>The runtime control surface stays deferred until a later task exposes real commands.</p>
    </section>
  {/if}
</section>

<style>
  .page-shell,
  header,
  .panel {
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
</style>
