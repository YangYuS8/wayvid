<script lang="ts">
  import { onMount } from 'svelte';
  import ItemCard from '$lib/components/ItemCard.svelte';
  import LibraryDetailPanel from '$lib/components/LibraryDetailPanel.svelte';
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

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : 'Unable to load the Library request.';

  let loading = false;
  let detailLoading = false;
  let pageError: string | null = null;
  let detailError: string | null = null;
  let detailRequestToken = 0;

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

<section class="page-shell">
  <header>
    <div>
      <p class="eyebrow">Library</p>
      <h1>Local projection snapshot</h1>
      <p>Keep the page thin: render the cached projection and request detail only for the current selection.</p>
    </div>
  </header>

  {#if pageError}
    <p class="message error">{pageError}</p>
  {/if}

  <div class="layout">
    <section>
      {#if loading && !$pageCache.library.snapshot}
        <p>Loading Library snapshot...</p>
      {:else if $pageCache.library.snapshot?.items.length}
        <div class="item-grid">
          {#each $pageCache.library.snapshot.items as item}
            <button
              type="button"
              class="item-button"
              aria-pressed={$pageCache.library.snapshot.selectedItemId === item.id}
              on:click={() => selectItem(item.id)}
            >
              <ItemCard
                title={item.title}
                itemType={item.itemType}
                coverPath={item.coverPath}
                primaryBadge={item.source}
                selected={$pageCache.library.snapshot.selectedItemId === item.id}
              />
            </button>
          {/each}
        </div>
      {:else}
        <p>No Library items are available in the current snapshot.</p>
      {/if}
    </section>

    <LibraryDetailPanel detail={$pageCache.library.detail} loading={detailLoading} error={detailError} />
  </div>
</section>

<style>
  .page-shell,
  header,
  .layout,
  .item-grid {
    display: grid;
    gap: 1.1rem;
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

  h1 {
    margin-top: 0.2rem;
    font-size: clamp(1.8rem, 4vw, 2.4rem);
  }

  .layout {
    grid-template-columns: minmax(0, 1.4fr) minmax(280px, 0.9fr);
    align-items: start;
  }

  .item-grid {
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  }

  .item-button {
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
    cursor: pointer;
    border-radius: 18px;
  }

  .item-button:focus-visible {
    outline: 3px solid #0f5f9a;
    outline-offset: 4px;
  }

  .message {
    padding: 0.85rem 1rem;
    border-radius: 14px;
  }

  .message.error {
    background: rgba(160, 98, 23, 0.12);
  }

  @media (max-width: 900px) {
    .layout {
      grid-template-columns: 1fr;
    }
  }
</style>
