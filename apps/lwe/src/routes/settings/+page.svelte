<script lang="ts">
  import { onMount } from 'svelte';
  import { loadSettingsPage } from '$lib/ipc';
  import { needsPageLoad, pageCache, setCurrentPage, setSettingsSnapshot } from '$lib/stores/ui';

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : 'Unable to load the Settings snapshot.';

  let loading = false;
  let pageError: string | null = null;

  const ensurePage = async () => {
    if (!needsPageLoad('settings')) {
      return;
    }

    loading = true;
    pageError = null;

    try {
      setSettingsSnapshot(await loadSettingsPage());
    } catch (error) {
      pageError = readError(error);
    } finally {
      loading = false;
    }
  };

  onMount(() => {
    setCurrentPage('settings');
    void ensurePage();
  });
</script>

<section class="page-shell">
  <header>
    <p class="eyebrow">Settings</p>
    <h1>Thin shell settings</h1>
    <p>Show the Rust-owned settings snapshot and avoid guessing preferences that the backend does not own yet.</p>
  </header>

  {#if pageError}
    <p class="message error" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !$pageCache.settings.snapshot}
    <p role="status" aria-live="polite">Loading Settings snapshot...</p>
  {:else if $pageCache.settings.snapshot}
    <section class="panel">
      <p>Language: {$pageCache.settings.snapshot.language}</p>
      <p>Theme: {$pageCache.settings.snapshot.theme}</p>
      <p>Steam required: {$pageCache.settings.snapshot.steamRequired ? 'yes' : 'no'}</p>
      <p>Snapshot stale: {$pageCache.settings.snapshot.stale ? 'yes' : 'no'}</p>
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
