<script lang="ts">
  import { Button } from '$lib/ui/button';
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
  import CompatibilityPanel from '$lib/components/CompatibilityPanel.svelte';
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { WorkshopItemDetail } from '$lib/types';

  export let detail: WorkshopItemDetail | null = null;
  export let loading = false;
  export let error: string | null = null;
  export let openInSteam: (() => Promise<void>) | null = null;
</script>

<Card class="grid gap-5 border-slate-200/80 bg-white/95 p-5 shadow-[0_24px_60px_rgba(15,23,42,0.08)] sm:p-6">
  {#if loading}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">Loading item details…</p>
  {:else if error}
    <p
      class="rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900"
      role="alert"
      aria-live="assertive"
    >
      {error}
    </p>
  {:else if detail}
    <div class="grid gap-5 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)] lg:items-start">
      <CoverImage coverPath={detail.coverPath} label={detail.title} />

      <div class="grid min-w-0 gap-5">
        <div class="grid gap-3">
          <div class="grid gap-2">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              Workshop item
            </p>
            <h2 class="text-2xl font-semibold text-slate-950">{detail.title}</h2>
          </div>

          <div class="flex flex-wrap gap-2">
            <StatusBadge label={detail.compatibility.badge} />
            <StatusBadge label={detail.syncStatus} />
            <StatusBadge label={detail.itemType} />
          </div>
        </div>

        <CompatibilityPanel compatibility={detail.compatibility} />

        <Separator class="bg-slate-200/80" />

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="grid gap-2 rounded-2xl border border-slate-200/80 bg-slate-50/60 p-4">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              Description
            </p>
            <p class="text-sm leading-6 text-slate-700">
              {detail.description ?? 'No description is available for this Workshop item yet.'}
            </p>
          </div>

          <div class="grid gap-2 rounded-2xl border border-slate-200/80 bg-slate-50/60 p-4">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Tags</p>
            <p class="text-sm leading-6 text-slate-700">
              {detail.tags.length > 0 ? detail.tags.join(' • ') : 'No tags are attached to this item.'}
            </p>
          </div>
        </div>

        <Button class="w-fit" onclick={() => openInSteam?.()} disabled={!openInSteam}>
          Open In Steam
        </Button>
      </div>
    </div>
  {:else}
    <div class="grid gap-2 rounded-2xl border border-dashed border-slate-300 bg-slate-50/60 p-5 text-sm text-slate-600">
      <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Workshop detail</p>
      <p>Select a Workshop item to inspect its current detail payload.</p>
    </div>
  {/if}
</Card>
