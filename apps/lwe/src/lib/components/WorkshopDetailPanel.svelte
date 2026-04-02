<script lang="ts">
  import { Button } from '$lib/ui/button';
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
  import CompatibilityPanel from '$lib/components/CompatibilityPanel.svelte';
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import { copy, getCompatibilityBadgeLabel, getItemTypeLabel, getWorkshopSyncStatusLabel } from '$lib/i18n';
  import type { WorkshopItemDetail } from '$lib/types';

  export let detail: WorkshopItemDetail | null = null;
  export let loading = false;
  export let error: string | null = null;
  export let openInSteam: (() => Promise<void>) | null = null;

  $: workshopDetailCopy = $copy.components.workshopDetail;
  $: detailCompatibilityLabel = detail ? getCompatibilityBadgeLabel($copy, detail.compatibility.badge) : '';
  $: detailItemTypeLabel = detail ? getItemTypeLabel($copy, detail.itemType) : '';
  $: detailSyncStatusLabel = detail ? getWorkshopSyncStatusLabel($copy, detail.syncStatus) : '';
</script>

<Card class="lwe-panel">
  {#if loading}
    <div class="lwe-subpanel gap-3" role="status" aria-live="polite">
      <p class="lwe-eyebrow">{workshopDetailCopy.title}</p>
      <p class="text-sm leading-6 text-slate-600">{workshopDetailCopy.loading}</p>
    </div>
  {:else if error}
    <div class="lwe-subpanel gap-3">
      <p class="lwe-eyebrow">{workshopDetailCopy.title}</p>
      <p class="lwe-warning-banner lwe-wrap-safe" role="alert" aria-live="assertive">
        {error}
      </p>
    </div>
  {:else if detail}
    <div class="grid min-w-0 gap-4" data-detail-layout="compact-vertical">
      <section class="grid gap-3.5" data-detail-section="header">
        <div class="grid gap-2">
          <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
            {workshopDetailCopy.itemTitle}
          </p>
          <h2 class="lwe-heading-lg lwe-wrap-safe">{detail.title}</h2>
        </div>

        <div class="flex flex-wrap gap-2">
          <StatusBadge label={detailCompatibilityLabel} />
          <StatusBadge label={detailSyncStatusLabel} />
          <StatusBadge label={detailItemTypeLabel} />
        </div>
      </section>

      <section class="lwe-subpanel gap-3" data-detail-section="quick-status">
        <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">{workshopDetailCopy.quickStatus}</p>
        <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
          {workshopDetailCopy.syncStatusPrefix} {detailSyncStatusLabel}. {workshopDetailCopy.compatibilityPrefix} {detail.compatibility.summaryCopy}.
        </p>
      </section>

      <section class="lwe-subpanel gap-3.5" data-detail-section="actions">
        <div class="grid gap-1.5">
          <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">{workshopDetailCopy.actions}</p>
          <h3 class="text-base font-semibold tracking-tight text-slate-950">{workshopDetailCopy.openSourcePage}</h3>
          <p class="text-sm leading-6 text-slate-600">
            {workshopDetailCopy.openSourceDescription}
          </p>
        </div>

        <Button class="w-fit" onclick={() => openInSteam?.()} disabled={!openInSteam}>
          {workshopDetailCopy.openInSteam}
        </Button>
      </section>

      <section class="grid gap-3" data-detail-section="cover">
        <div class="grid max-w-sm gap-2">
          <div class="grid gap-2">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">{workshopDetailCopy.cover}</p>
            <p class="text-sm leading-6 text-slate-600">
              {workshopDetailCopy.coverDescription}
            </p>
          </div>
          <CoverImage coverPath={detail.coverPath} label={detail.title} />
        </div>
      </section>

      <section data-detail-section="compatibility">
        <CompatibilityPanel compatibility={detail.compatibility} badgeLabel={detailCompatibilityLabel} />
      </section>

      <Separator class="bg-slate-200/80" />

      <section class="grid gap-4 sm:grid-cols-2" data-detail-section="metadata">
          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              {workshopDetailCopy.description}
            </p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
              {detail.description ?? workshopDetailCopy.noDescription}
            </p>
          </div>

          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">{workshopDetailCopy.tags}</p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
              {detail.tags.length > 0 ? detail.tags.join(' • ') : workshopDetailCopy.noTags}
            </p>
          </div>

          <div class="lwe-subpanel sm:col-span-2">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">{workshopDetailCopy.syncState}</p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">{detailSyncStatusLabel}</p>
          </div>
      </section>
    </div>
  {:else}
    <div class="lwe-subpanel gap-3 border-dashed" role="status" aria-live="polite">
      <p class="lwe-eyebrow">{workshopDetailCopy.title}</p>
      <p class="text-sm leading-6 text-slate-600">{workshopDetailCopy.empty}</p>
    </div>
  {/if}
</Card>
