<script lang="ts">
  import { Card } from '$lib/ui/card';
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { CompatibilitySummaryModel } from '$lib/types';

  export let title: string;
  export let itemType: string;
  export let coverPath: string | null = null;
  export let compatibility: CompatibilitySummaryModel;
  export let selected = false;
  export let assignedMonitorLabels: string[] = [];
</script>

<Card
  class={`group grid gap-4 overflow-hidden border-slate-200/80 bg-white/95 p-3 shadow-[0_16px_40px_rgba(15,23,42,0.08)] transition-transform duration-150 hover:-translate-y-0.5 hover:shadow-[0_20px_48px_rgba(15,23,42,0.12)] ${selected ? 'border-slate-900 ring-1 ring-slate-900/10' : ''}`}
>
  <CoverImage {coverPath} label={title} />

  <div class="grid min-w-0 gap-3 px-1 pb-1">
    <div class="grid gap-2">
      <div class="flex flex-wrap items-center gap-2">
        <StatusBadge label={compatibility.badge} />
        <span class="rounded-full border border-slate-200 bg-slate-50 px-2.5 py-1 text-[0.68rem] font-medium uppercase tracking-[0.18em] text-slate-500">
          {itemType}
        </span>
      </div>

      <div class="grid gap-1">
        <h3 class="line-clamp-2 text-base font-semibold text-slate-950">{title}</h3>
        <p class="text-sm leading-6 text-slate-600">{compatibility.summaryCopy}</p>
      </div>
    </div>

    {#if assignedMonitorLabels.length > 0}
      <div class="grid gap-1 rounded-lg border border-slate-200/80 bg-slate-50/80 px-3 py-2">
        <p class="text-[0.68rem] font-semibold uppercase tracking-[0.18em] text-slate-500">
          Assigned to
        </p>
        <p class="text-sm text-slate-700">{assignedMonitorLabels.join(' • ')}</p>
      </div>
    {/if}
  </div>
</Card>
