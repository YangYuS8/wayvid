<script lang="ts">
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
  import CompatibilityPanel from '$lib/components/CompatibilityPanel.svelte';
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { LibraryItemDetail, LibraryPageSnapshot } from '$lib/types';
  import { resolveLibraryAvailabilityIssues } from '../../routes/library/page-state';

  export let detail: LibraryItemDetail | null = null;
  export let snapshot: LibraryPageSnapshot | null = null;
  export let loading = false;
  export let error: string | null = null;

  $: availabilitySource = detail ?? snapshot;
  $: issueMessages = availabilitySource ? resolveLibraryAvailabilityIssues(availabilitySource) : [];
  $: assignedMonitorLabels = detail?.assignedMonitorLabels ?? [];
</script>

<Card class="lwe-panel">
  {#if loading}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">Loading item details…</p>
  {:else if error}
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">
      {error}
    </p>
  {:else if detail}
    <div class="grid gap-6 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)] lg:items-start">
      <CoverImage coverPath={detail.coverPath} label={detail.title} />

      <div class="grid min-w-0 gap-5">
        <div class="grid gap-3.5">
          <div class="grid gap-2">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              Library item
            </p>
            <h2 class="lwe-heading-lg lwe-wrap-safe">{detail.title}</h2>
          </div>

          <div class="flex flex-wrap gap-2">
            <StatusBadge label={detail.compatibility.badge} />
            <StatusBadge label={detail.source} />
            <StatusBadge label={detail.itemType} />
          </div>
        </div>

        {#if issueMessages.length}
          <div class="grid gap-2.5" aria-live="polite">
            {#each issueMessages as issue}
              <p class="lwe-info-banner">
                {issue}
              </p>
            {/each}
          </div>
        {/if}

        {#if assignedMonitorLabels.length > 0}
          <div class="lwe-subpanel" aria-live="polite">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              Assigned monitors
            </p>
            <p class="lwe-wrap-safe text-sm text-slate-700">{assignedMonitorLabels.join(' • ')}</p>
          </div>
        {/if}

        <CompatibilityPanel compatibility={detail.compatibility} />

        <Separator class="bg-slate-200/80" />

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              Description
            </p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
              {detail.description ?? 'No description available for this item yet.'}
            </p>
          </div>

          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Tags</p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
              {detail.tags.length > 0 ? detail.tags.join(' • ') : 'No tags are attached to this item.'}
            </p>
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div class="grid gap-4">
      {#if issueMessages.length}
        <div class="grid gap-2.5" aria-live="polite">
          {#each issueMessages as issue}
            <p class="lwe-info-banner">
              {issue}
            </p>
          {/each}
        </div>
      {/if}

      <div class="grid gap-2 rounded-[1.35rem] border border-dashed border-slate-300 bg-slate-50/60 p-5 text-sm text-slate-600">
        <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Library detail</p>
        <p>Select a Library item to inspect its current detail payload.</p>
      </div>
    </div>
  {/if}
</Card>
