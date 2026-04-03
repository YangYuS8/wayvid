<script lang="ts">
  import { onMount } from 'svelte';
  import type { InvalidatedPage } from '$lib/types';
  import ItemCard from '$lib/components/ItemCard.svelte';
  import LibraryDetailPanel from '$lib/components/LibraryDetailPanel.svelte';
  import PageHeader from '$lib/layout/PageHeader.svelte';
  import { copy, formatCopy } from '$lib/i18n';
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
  import { resolveLibraryApplyRefreshState, resolveLibraryPageState } from './page-state';

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : $copy.library.requestError;

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
  $: pageState = snapshot ? resolveLibraryPageState(snapshot, $copy.library) : null;
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

  const loadSelectedDetail = async (itemId: string) => {
    const requestToken = ++detailRequestToken;
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

  const refreshInvalidatedPages = async (invalidations: InvalidatedPage[]) => {
    let refreshedSelectedItemId = snapshot?.selectedItemId ?? null;
    let librarySnapshotRefreshSucceeded = true;
    const initialRefreshState = resolveLibraryApplyRefreshState({
      invalidations,
      selectedItemId: refreshedSelectedItemId
    });

    if (initialRefreshState.refreshLibrarySnapshot) {
      try {
        const refreshedSnapshot = await loadLibraryPage();
        setLibrarySnapshot(refreshedSnapshot);
        refreshedSelectedItemId = refreshedSnapshot.selectedItemId;
      } catch (error) {
        librarySnapshotRefreshSucceeded = false;

        const message = readError(error);
        if (snapshot) {
          applyError = message;
        } else {
          pageError = message;
        }
      }
    }

    const refreshState = resolveLibraryApplyRefreshState({
      invalidations,
      selectedItemId: refreshedSelectedItemId,
      librarySnapshotRefreshSucceeded
    });

    if (refreshState.refreshLibraryDetailId) {
      await loadSelectedDetail(refreshState.refreshLibraryDetailId);
    }

    if (refreshState.refreshDesktopSnapshot) {
      await ensureDesktopSnapshot();
    }
  };

  const selectItem = async (itemId: string) => {
    setSelectedItem('library', itemId);
    applyError = null;
    applyMessage = null;

    await loadSelectedDetail(itemId);
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
      await refreshInvalidatedPages(outcome.invalidations);
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
  <title>{$copy.library.pageTitle}</title>
</svelte:head>

<section class="grid gap-6">
  <PageHeader
    eyebrow={$copy.library.pageTitle}
    title={$copy.library.headerTitle}
    subtitle={$copy.library.headerSubtitle}
  />

  {#if pageError && !snapshot}
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !snapshot}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">{$copy.library.loading}</p>
  {:else if snapshot}
    <div class="grid gap-5 xl:grid-cols-[minmax(0,1.4fr)_minmax(300px,0.9fr)] xl:items-start">
      <section class="grid gap-4">
        {#if pageError}
          <p class="lwe-warning-banner" role="alert" aria-live="assertive">{pageError}</p>
        {/if}

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
                selectLabel={formatCopy($copy.library.selectItemLabel, { itemTitle: item.title })}
                onSelect={() => selectItem(item.id)}
                onApplyShortcut={() => selectItem(item.id)}
              />
            {/each}
          </div>
        {:else}
          <p class="text-sm leading-6 text-slate-600">
            {pageState?.emptyMessage ?? $copy.library.empty}
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
        {applyError}
        applyMessage={applyMessage}
        onApply={applySelectedItem}
        onMonitorChange={(monitorId) => {
          applyMonitorId = monitorId;
        }}
      />
    </div>
  {/if}
</section>
