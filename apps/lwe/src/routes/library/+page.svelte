<script lang="ts">
  import { onMount } from 'svelte';
  import ItemCard from '$lib/components/ItemCard.svelte';
  import LibraryDetailPanel from '$lib/components/LibraryDetailPanel.svelte';
  import PageHeader from '$lib/layout/PageHeader.svelte';
  import { loadLibraryItemDetail, loadLibraryPage } from '$lib/ipc';
  import {
    isSelectedItem,
    needsPageLoad,
    pageCache,
    setCurrentPage,
    setLibraryDetailIfSelected,
    setLibrarySnapshot,
    setSelectedItem
  } from '$lib/stores/ui';
  import { resolveLibraryPageState } from './page-state';

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : 'Unable to load the Library request.';

  let loading = false;
  let detailLoading = false;
  let pageError: string | null = null;
  let detailError: string | null = null;
  let detailRequestToken = 0;

  $: snapshot = $pageCache.library.snapshot;
  $: pageState = snapshot ? resolveLibraryPageState(snapshot) : null;

  const ensurePage = async () => {
    if (!needsPageLoad('library')) {
      return;
    }

    loading = true;
    pageError = null;

    try {
      setLibrarySnapshot(await loadLibraryPage());
    } catch (error) {
      pageError = readError(error);
    } finally {
      loading = false;
    }
  };

  const selectItem = async (itemId: string) => {
    const requestToken = ++detailRequestToken;

    setSelectedItem('library', itemId);
    detailLoading = true;
    detailError = null;

    try {
      const detail = await loadLibraryItemDetail(itemId);
      if (requestToken !== detailRequestToken) {
        return;
      }

      setLibraryDetailIfSelected(detail, itemId);
    } catch (error) {
      if (requestToken !== detailRequestToken || !isSelectedItem('library', itemId)) {
        return;
      }

      setLibraryDetailIfSelected(null, itemId);
      detailError = readError(error);
    } finally {
      if (requestToken === detailRequestToken) {
        detailLoading = false;
      }
    }
  };

  onMount(() => {
    setCurrentPage('library');
    void ensurePage();
  });
</script>

<svelte:head>
  <title>Library</title>
</svelte:head>

<section class="grid gap-6">
  <PageHeader
    eyebrow="Library"
    title="Your local library"
    subtitle="Browse the content you already own or have synchronized onto this machine."
  />

  {#if pageError}
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !snapshot}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">Loading Library snapshot…</p>
  {:else if snapshot}
    <div class="grid gap-5 xl:grid-cols-[minmax(0,1.4fr)_minmax(300px,0.9fr)] xl:items-start">
      <section class="grid gap-4">
        {#if pageState?.issueMessages.length}
          <div class="grid gap-2.5" aria-live="polite">
            {#each pageState.issueMessages as issue}
              <p class="lwe-info-banner">{issue}</p>
            {/each}
          </div>
        {/if}

        {#if snapshot.items.length}
          <div class="grid gap-4 [grid-template-columns:repeat(auto-fit,minmax(220px,1fr))]">
            {#each snapshot.items as item}
              <button
                type="button"
                class="rounded-[1.125rem] border-0 bg-transparent p-0 text-left transition duration-150 hover:-translate-y-0.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 focus-visible:ring-offset-4"
                aria-pressed={snapshot.selectedItemId === item.id}
                on:click={() => selectItem(item.id)}
              >
                <ItemCard
                  title={item.title}
                  itemType={item.itemType}
                  coverPath={item.coverPath}
                  compatibility={item.compatibility}
                  selected={snapshot.selectedItemId === item.id}
                  assignedMonitorLabels={item.assignedMonitorLabels ?? []}
                />
              </button>
            {/each}
          </div>
        {:else}
          <p class="text-sm leading-6 text-slate-600">
            {pageState?.emptyMessage ?? 'No Library items are available in the current snapshot.'}
          </p>
        {/if}
      </section>

      <LibraryDetailPanel
        detail={$pageCache.library.detail}
        snapshot={snapshot}
        loading={detailLoading}
        error={detailError}
      />
    </div>
  {/if}
</section>
