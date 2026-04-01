<script lang="ts">
  import { onMount } from 'svelte';
  import PageHeader from '$lib/layout/PageHeader.svelte';
  import { Card } from '$lib/ui/card';
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

<svelte:head>
  <title>Settings</title>
</svelte:head>

<section class="page">
  <PageHeader
    eyebrow="Settings"
    title="Thin shell settings"
    subtitle="Show the Rust-owned settings snapshot and avoid guessing preferences that the backend does not own yet."
  />

  {#if pageError}
    <p class="message error" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !$pageCache.settings.snapshot}
    <p role="status" aria-live="polite">Loading Settings snapshot…</p>
  {:else if $pageCache.settings.snapshot}
    <Card class="panel border-slate-200/80 bg-white/95 p-5 shadow-[0_24px_60px_rgba(15,23,42,0.08)] sm:p-6">
      <div class="setting-row">
        <p class="label">Language</p>
        <p class="value">{$pageCache.settings.snapshot.language}</p>
      </div>

      <div class="setting-row">
        <p class="label">Theme</p>
        <p class="value">{$pageCache.settings.snapshot.theme}</p>
      </div>

      <div class="setting-row">
        <p class="label">Steam required</p>
        <p class="value">{$pageCache.settings.snapshot.steamRequired ? 'yes' : 'no'}</p>
      </div>

      <div class="setting-row">
        <p class="label">Snapshot stale</p>
        <p class="value">{$pageCache.settings.snapshot.stale ? 'yes' : 'no'}</p>
      </div>
    </Card>
  {/if}
</section>

<style>
  .page,
  .setting-row {
    display: grid;
    gap: 1rem;
  }

  p {
    margin: 0;
  }

  .setting-row {
    grid-template-columns: minmax(0, 180px) minmax(0, 1fr);
    align-items: baseline;
    gap: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid rgba(148, 163, 184, 0.18);
  }

  .setting-row:last-child {
    padding-bottom: 0;
    border-bottom: 0;
  }

  .label {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: #526272;
  }

  .value {
    color: #0f172a;
  }

  .message.error {
    padding: 0.85rem 1rem;
    border-radius: 14px;
    background: rgba(160, 98, 23, 0.12);
  }

  @media (max-width: 640px) {
    .setting-row {
      grid-template-columns: 1fr;
      gap: 0.35rem;
    }
  }
</style>
