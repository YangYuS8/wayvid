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

<section class="grid gap-6">
  <PageHeader
    eyebrow="Settings"
    title="Thin shell settings"
    subtitle="Show the Rust-owned settings snapshot and avoid guessing preferences that the backend does not own yet."
  />

  {#if pageError}
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !$pageCache.settings.snapshot}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">Loading Settings snapshot…</p>
  {:else if $pageCache.settings.snapshot}
    <Card class="lwe-panel gap-0">
      <div class="grid gap-1 border-b border-slate-200/80 pb-4 sm:grid-cols-[minmax(0,180px)_minmax(0,1fr)] sm:gap-4">
        <p class="lwe-eyebrow">Language</p>
        <p class="text-sm leading-6 text-slate-950">{$pageCache.settings.snapshot.language}</p>
      </div>

      <div class="grid gap-1 border-b border-slate-200/80 py-4 sm:grid-cols-[minmax(0,180px)_minmax(0,1fr)] sm:gap-4">
        <p class="lwe-eyebrow">Theme</p>
        <p class="text-sm leading-6 text-slate-950">{$pageCache.settings.snapshot.theme}</p>
      </div>

      <div class="grid gap-1 border-b border-slate-200/80 py-4 sm:grid-cols-[minmax(0,180px)_minmax(0,1fr)] sm:gap-4">
        <p class="lwe-eyebrow">Steam required</p>
        <p class="text-sm leading-6 text-slate-950">{$pageCache.settings.snapshot.steamRequired ? 'yes' : 'no'}</p>
      </div>

      <div class="grid gap-1 pt-4 sm:grid-cols-[minmax(0,180px)_minmax(0,1fr)] sm:gap-4">
        <p class="lwe-eyebrow">Snapshot stale</p>
        <p class="text-sm leading-6 text-slate-950">{$pageCache.settings.snapshot.stale ? 'yes' : 'no'}</p>
      </div>
    </Card>
  {/if}
</section>
