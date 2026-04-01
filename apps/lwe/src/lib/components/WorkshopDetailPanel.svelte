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

<Card class="lwe-panel">
  {#if loading}
    <div class="lwe-subpanel gap-3" role="status" aria-live="polite">
      <p class="lwe-eyebrow">Workshop detail</p>
      <p class="text-sm leading-6 text-slate-600">Loading item details…</p>
    </div>
  {:else if error}
    <div class="lwe-subpanel gap-3">
      <p class="lwe-eyebrow">Workshop detail</p>
      <p class="lwe-warning-banner lwe-wrap-safe" role="alert" aria-live="assertive">
        {error}
      </p>
    </div>
  {:else if detail}
    <div class="grid min-w-0 gap-4" data-detail-layout="compact-vertical">
      <section class="grid gap-3.5" data-detail-section="header">
        <div class="grid gap-2">
          <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
            Workshop item
          </p>
          <h2 class="lwe-heading-lg lwe-wrap-safe">{detail.title}</h2>
        </div>

        <div class="flex flex-wrap gap-2">
          <StatusBadge label={detail.compatibility.badge} />
          <StatusBadge label={detail.syncStatus} />
          <StatusBadge label={detail.itemType} />
        </div>
      </section>

      <section class="lwe-subpanel gap-3" data-detail-section="quick-status">
        <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Quick status</p>
        <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
          Sync status: {detail.syncStatus}. Compatibility: {detail.compatibility.summaryCopy}.
        </p>
      </section>

      <section class="lwe-subpanel gap-3.5" data-detail-section="actions">
        <div class="grid gap-1.5">
          <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Actions</p>
          <h3 class="text-base font-semibold tracking-tight text-slate-950">Open source page</h3>
          <p class="text-sm leading-6 text-slate-600">
            Jump to Steam for subscription, comments, and other Workshop context.
          </p>
        </div>

        <Button class="w-fit" onclick={() => openInSteam?.()} disabled={!openInSteam}>
          Open In Steam
        </Button>
      </section>

      <section class="grid gap-3" data-detail-section="cover">
        <div class="grid max-w-sm gap-2">
          <div class="grid gap-2">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Cover</p>
            <p class="text-sm leading-6 text-slate-600">
              Compact artwork preview for quick scanning inside the denser Workshop detail flow.
            </p>
          </div>
          <CoverImage coverPath={detail.coverPath} label={detail.title} />
        </div>
      </section>

      <section data-detail-section="compatibility">
        <CompatibilityPanel compatibility={detail.compatibility} />
      </section>

      <Separator class="bg-slate-200/80" />

      <section class="grid gap-4 sm:grid-cols-2" data-detail-section="metadata">
          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              Description
            </p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
              {detail.description ?? 'No description is available for this Workshop item yet.'}
            </p>
          </div>

          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Tags</p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
              {detail.tags.length > 0 ? detail.tags.join(' • ') : 'No tags are attached to this item.'}
            </p>
          </div>

          <div class="lwe-subpanel sm:col-span-2">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Sync state</p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">{detail.syncStatus}</p>
          </div>
      </section>
    </div>
  {:else}
    <div class="lwe-subpanel gap-3 border-dashed" role="status" aria-live="polite">
      <p class="lwe-eyebrow">Workshop detail</p>
      <p class="text-sm leading-6 text-slate-600">Select a Workshop item to inspect its current detail payload.</p>
    </div>
  {/if}
</Card>
