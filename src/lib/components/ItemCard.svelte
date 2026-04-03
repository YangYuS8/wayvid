<script lang="ts">
  import { Card } from '$lib/ui/card';
  import CoverImage from '$lib/components/CoverImage.svelte';
  import ItemActionsMenu from '$lib/components/ItemActionsMenu.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import { copy, getCompatibilityBadgeLabel, getItemTypeLabel } from '$lib/i18n';
  import type { CompatibilitySummaryModel, ItemType } from '$lib/types';

  export let title: string;
  export let itemType: ItemType;
  export let coverPath: string | null = null;
  export let compatibility: CompatibilitySummaryModel;
  export let selected = false;
  export let assignedMonitorLabels: string[] = [];
  export let selectLabel: string | null = null;
  export let onSelect: (() => void) | undefined = undefined;
  export let onApplyShortcut: (() => void) | undefined = undefined;

  $: itemTypeLabel = getItemTypeLabel($copy, itemType);
  $: compatibilityBadgeLabel = getCompatibilityBadgeLabel($copy, compatibility.badge);
</script>

<Card
  class={`relative lwe-panel-compact group transition duration-150 hover:-translate-y-0.5 hover:border-slate-300/80 hover:shadow-[0_24px_56px_rgba(15,23,42,0.12)] ${selected ? 'border-slate-900/80 ring-1 ring-slate-900/10' : ''}`}
>
  {#if onSelect && selectLabel}
    <button
      type="button"
      class="absolute inset-0 z-10 rounded-[1.125rem] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 focus-visible:ring-offset-4"
      aria-label={selectLabel}
      aria-pressed={selected}
      onclick={onSelect}
    ></button>
  {/if}

  {#if onApplyShortcut}
    <div class="absolute right-3 top-3 z-20">
      <ItemActionsMenu itemTitle={title} {onApplyShortcut} />
    </div>
  {/if}

  <div class={`grid gap-4 ${onSelect ? 'pointer-events-none relative z-0' : ''}`}>
    <CoverImage {coverPath} label={title} />

    <div class="grid min-w-0 gap-3.5 px-1 pb-1">
      <div class="grid gap-2.5">
        <div class="flex flex-wrap items-center gap-2">
          <StatusBadge label={compatibilityBadgeLabel} />
          <span class="lwe-pill-label">
            {itemTypeLabel}
          </span>
        </div>

        <div class="grid gap-1.5">
          <h3 class="line-clamp-2 text-base font-semibold leading-6 text-slate-950">{title}</h3>
          <p class="lwe-body-copy">{compatibility.summaryCopy}</p>
        </div>
      </div>

      {#if assignedMonitorLabels.length > 0}
        <div class="lwe-subpanel gap-1.5 px-3.5 py-3">
          <p class="text-[0.68rem] font-semibold uppercase tracking-[0.18em] text-slate-500">
            {$copy.components.itemCard.assignedTo}
          </p>
          <p class="text-sm text-slate-700">{assignedMonitorLabels.join(' • ')}</p>
        </div>
      {/if}
    </div>
  </div>
</Card>
