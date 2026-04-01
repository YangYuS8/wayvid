<script lang="ts">
  import { onMount } from 'svelte';
  import ItemCard from '$lib/components/ItemCard.svelte';
  import WorkshopDetailPanel from '$lib/components/WorkshopDetailPanel.svelte';
  import PageHeader from '$lib/layout/PageHeader.svelte';
  import { Button } from '$lib/ui/button';
  import { resolveWorkshopRefreshState } from './page-state';
  import {
    loadWorkshopItemDetail,
    loadWorkshopPage,
    openWorkshopInSteam,
    refreshWorkshopCatalog
  } from '$lib/ipc';
  import {
    applyInvalidations,
    isSelectedItem,
    needsPageLoad,
    pageCache,
    setCurrentPage,
    setPageStale,
    setSelectedItem,
    setWorkshopDetail,
    setWorkshopDetailIfSelected,
    setWorkshopSnapshot
  } from '$lib/stores/ui';

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : 'Unable to complete the Workshop request.';

  let loading = false;
  let detailLoading = false;
  let pageError: string | null = null;
  let detailError: string | null = null;
  let actionMessage: string | null = null;
  let detailRequestToken = 0;

  const ensurePage = async () => {
    if (!needsPageLoad('workshop')) {
      return;
    }

    loading = true;
    pageError = null;

    try {
      setWorkshopSnapshot(await loadWorkshopPage());
    } catch (error) {
      pageError = readError(error);
    } finally {
      loading = false;
    }
  };

  const selectItem = async (itemId: string) => {
    const requestToken = ++detailRequestToken;

    setSelectedItem('workshop', itemId);
    detailLoading = true;
    detailError = null;

    try {
      const detail = await loadWorkshopItemDetail(itemId);
      if (requestToken !== detailRequestToken) {
        return;
      }

      setWorkshopDetailIfSelected(detail, itemId);
    } catch (error) {
      if (requestToken !== detailRequestToken || !isSelectedItem('workshop', itemId)) {
        return;
      }

      setWorkshopDetailIfSelected(null, itemId);
      detailError = readError(error);
    } finally {
      if (requestToken === detailRequestToken) {
        detailLoading = false;
      }
    }
  };

  const refreshPage = async () => {
    loading = true;
    pageError = null;
    actionMessage = null;

    try {
      const previousSelection = $pageCache.workshop.snapshot?.selectedItemId ?? null;
      const outcome = await refreshWorkshopCatalog();
      const currentSelection = $pageCache.workshop.snapshot?.selectedItemId ?? null;
      const availableItemIds = outcome.currentUpdate?.items.map((item: { id: string }) => item.id) ?? [];
      const refreshState = resolveWorkshopRefreshState({
        previousSelection,
        currentSelection,
        availableItemIds,
        detailLoading,
        detailRequestToken,
        detailError
      });

      if (outcome.currentUpdate) {
        setWorkshopSnapshot({
          ...outcome.currentUpdate,
          selectedItemId: refreshState.nextSelection
        });
      } else {
        setPageStale('workshop', true);
      }

      ({ detailLoading, detailRequestToken, detailError } = refreshState);

      applyInvalidations(outcome.invalidations);
      actionMessage = outcome.message;

      if (refreshState.nextSelection && $pageCache.workshop.snapshot?.selectedItemId === refreshState.nextSelection) {
        await selectItem(refreshState.nextSelection);
      } else {
        setWorkshopDetail(null);
      }
    } catch (error) {
      pageError = readError(error);
    } finally {
      loading = false;
    }
  };

  const openSelectedInSteam = async () => {
    const selectedId = $pageCache.workshop.detail?.id;
    if (!selectedId) {
      return;
    }

    detailError = null;

    try {
      const outcome = await openWorkshopInSteam(selectedId);
      actionMessage = outcome.message;
      applyInvalidations(outcome.invalidations);
    } catch (error) {
      detailError = readError(error);
    }
  };

  onMount(() => {
    setCurrentPage('workshop');
    void ensurePage();
  });
</script>

<svelte:head>
  <title>Workshop</title>
</svelte:head>

<section class="grid gap-6">
  <PageHeader
    eyebrow="Workshop"
    title="Steam-backed snapshot"
    subtitle="Render the cached Rust snapshot, fetch details on selection, and keep stale invalidations local."
  >
    {#snippet actions()}
      <Button variant="secondary" onclick={refreshPage} disabled={loading}>Refresh Catalog</Button>
    {/snippet}
  </PageHeader>

  {#if pageError}
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !$pageCache.workshop.snapshot}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">Loading Workshop snapshot…</p>
  {:else if $pageCache.workshop.snapshot}
    {#if actionMessage}
      <p class="lwe-info-banner" role="status" aria-live="polite">{actionMessage}</p>
    {/if}

    <div class="grid gap-5 xl:grid-cols-[minmax(0,1.4fr)_minmax(300px,0.9fr)] xl:items-start">
      <section class="grid gap-4">
        {#if $pageCache.workshop.snapshot.items.length}
          <div class="grid gap-4 [grid-template-columns:repeat(auto-fit,minmax(220px,1fr))]">
            {#each $pageCache.workshop.snapshot.items as item}
              <button
                type="button"
                class="rounded-[1.125rem] border-0 bg-transparent p-0 text-left transition duration-150 hover:-translate-y-0.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 focus-visible:ring-offset-4"
                aria-pressed={$pageCache.workshop.snapshot.selectedItemId === item.id}
                on:click={() => selectItem(item.id)}
              >
                <ItemCard
                  title={item.title}
                  itemType={item.itemType}
                  coverPath={item.coverPath}
                  compatibility={item.compatibility}
                  selected={$pageCache.workshop.snapshot.selectedItemId === item.id}
                />
              </button>
            {/each}
          </div>
        {:else}
          <p class="text-sm leading-6 text-slate-600">No Workshop items are available in the current snapshot.</p>
        {/if}
      </section>

      <WorkshopDetailPanel
        detail={$pageCache.workshop.detail}
        loading={detailLoading}
        error={detailError}
        openInSteam={openSelectedInSteam}
      />
    </div>
  {/if}
</section>
