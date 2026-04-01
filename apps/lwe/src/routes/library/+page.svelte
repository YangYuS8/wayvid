<script lang="ts">
  import { onMount } from 'svelte';
  import ItemCard from '$lib/components/ItemCard.svelte';
  import LibraryDetailPanel from '$lib/components/LibraryDetailPanel.svelte';
  import PageHeader from '$lib/layout/PageHeader.svelte';
  import { applyLibraryItemToMonitor, loadDesktopPage, loadLibraryItemDetail, loadLibraryPage } from '$lib/ipc';
  import {
    applyInvalidations,
    isSelectedItem,
    needsPageLoad,
    pageCache,
    setCurrentPage,
    setDesktopSnapshot,
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
  let applyError: string | null = null;
  let applyMessage: string | null = null;
  let applyLoading = false;
  let applyMonitorId = '';
  let detailRequestToken = 0;

  $: snapshot = $pageCache.library.snapshot;
  $: pageState = snapshot ? resolveLibraryPageState(snapshot) : null;
  $: desktopSnapshot = $pageCache.desktop.snapshot;
  $: availableMonitors = desktopSnapshot?.monitors ?? [];
  $: selectedDetail = $pageCache.library.detail;
  $: {
    if (!availableMonitors.length) {
      applyMonitorId = '';
    } else if (!availableMonitors.some((monitor) => monitor.monitorId === applyMonitorId)) {
      applyMonitorId = availableMonitors[0]?.monitorId ?? '';
    }
  }

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

  const ensureDesktopSnapshot = async () => {
    if (!needsPageLoad('desktop')) {
      return;
    }

    try {
      setDesktopSnapshot(await loadDesktopPage());
    } catch {
      // Keep the primary action visible even if monitor discovery fails.
    }
  };

  const selectItem = async (itemId: string) => {
    const requestToken = ++detailRequestToken;

    setSelectedItem('library', itemId);
    detailLoading = true;
    detailError = null;
    applyError = null;
    applyMessage = null;

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

  const applySelectedItem = async () => {
    if (!selectedDetail || !applyMonitorId) {
      return;
    }

    applyLoading = true;
    applyError = null;
    applyMessage = null;

    try {
      const outcome = await applyLibraryItemToMonitor(applyMonitorId, selectedDetail.id);
      applyMessage = outcome.message;
      applyInvalidations(outcome.invalidations);
      await ensureDesktopSnapshot();
    } catch (error) {
      applyError = readError(error);
    } finally {
      applyLoading = false;
    }
  };

  onMount(() => {
    setCurrentPage('library');
    void ensurePage();
    void ensureDesktopSnapshot();
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
  {:else if applyError}
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">{applyError}</p>
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
              <ItemCard
                title={item.title}
                itemType={item.itemType}
                coverPath={item.coverPath}
                compatibility={item.compatibility}
                selected={snapshot.selectedItemId === item.id}
                assignedMonitorLabels={item.assignedMonitorLabels ?? []}
                selectLabel={`Select ${item.title}`}
                onSelect={() => selectItem(item.id)}
                onApplyShortcut={() => selectItem(item.id)}
              />
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
        monitors={availableMonitors}
        selectedMonitorId={applyMonitorId}
        applyDisabled={!selectedDetail || !applyMonitorId}
        applying={applyLoading}
        applyMessage={applyMessage}
        onApply={applySelectedItem}
        onMonitorChange={(monitorId) => {
          applyMonitorId = monitorId;
        }}
      />
    </div>
  {/if}
</section>
