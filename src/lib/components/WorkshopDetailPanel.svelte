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
      <p class="text-sm leading-6 text-muted-foreground">{workshopDetailCopy.loading}</p>
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
          <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-muted-foreground">
            {workshopDetailCopy.itemTitle}
          </p>
          <h2 class="lwe-heading-lg lwe-wrap-safe">{detail.title}</h2>
        </div>

        <div class="flex flex-wrap gap-2">
          <StatusBadge label={detailCompatibilityLabel} variantKey={detail.compatibility.badge} />
          <StatusBadge label={detailSyncStatusLabel} variantKey={detail.syncStatus} />
          <StatusBadge label={detailItemTypeLabel} variantKey={detail.itemType} />
        </div>
      </section>

      <section class="lwe-subpanel gap-3" data-detail-section="quick-status">
        <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-muted-foreground">{workshopDetailCopy.quickStatus}</p>
        <p class="lwe-wrap-safe text-sm leading-6 text-foreground/85">
          {workshopDetailCopy.syncStatusPrefix} {detailSyncStatusLabel}. {workshopDetailCopy.compatibilityPrefix} {detail.compatibility.summaryCopy}.
        </p>
      </section>

      <section class="lwe-subpanel gap-3.5" data-detail-section="actions">
        <div class="grid gap-1.5">
          <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-muted-foreground">{workshopDetailCopy.actions}</p>
          <h3 class="text-base font-semibold tracking-tight text-foreground">{workshopDetailCopy.openSourcePage}</h3>
          <p class="text-sm leading-6 text-muted-foreground">
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
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-muted-foreground">{workshopDetailCopy.cover}</p>
            <p class="text-sm leading-6 text-muted-foreground">
              {workshopDetailCopy.coverDescription}
            </p>
          </div>
          <CoverImage coverPath={detail.coverPath} label={detail.title} />
        </div>
      </section>

      <section data-detail-section="compatibility">
        <CompatibilityPanel
          compatibility={detail.compatibility}
          badgeLabel={detailCompatibilityLabel}
          badgeVariantKey={detail.compatibility.badge}
        />
      </section>

      <Separator class="bg-border/80" />

      <section class="grid gap-4 sm:grid-cols-2" data-detail-section="metadata">
          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-muted-foreground">
              {workshopDetailCopy.description}
            </p>
            <p class="lwe-wrap-safe text-sm leading-6 text-foreground/85">
              {detail.description ?? workshopDetailCopy.noDescription}
            </p>
          </div>

          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-muted-foreground">{workshopDetailCopy.tags}</p>
            <p class="lwe-wrap-safe text-sm leading-6 text-foreground/85">
              {detail.tags.length > 0 ? detail.tags.join(' • ') : workshopDetailCopy.noTags}
            </p>
          </div>

          <div class="lwe-subpanel sm:col-span-2">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-muted-foreground">{workshopDetailCopy.syncState}</p>
            <p class="lwe-wrap-safe text-sm leading-6 text-foreground/85">{detailSyncStatusLabel}</p>
          </div>
      </section>
    </div>
  {:else}
    <div class="lwe-subpanel gap-3 border-dashed" role="status" aria-live="polite">
      <p class="lwe-eyebrow">{workshopDetailCopy.title}</p>
      <p class="text-sm leading-6 text-muted-foreground">{workshopDetailCopy.empty}</p>
    </div>
  {/if}
</Card>
