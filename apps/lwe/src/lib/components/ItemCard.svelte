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
  class={`group grid gap-4 overflow-hidden rounded-[1.5rem] border-slate-200/70 bg-white/95 p-3.5 shadow-[0_18px_44px_rgba(15,23,42,0.08)] transition duration-150 hover:-translate-y-0.5 hover:border-slate-300/80 hover:shadow-[0_24px_56px_rgba(15,23,42,0.12)] ${selected ? 'border-slate-900/80 ring-1 ring-slate-900/10' : ''}`}
>
  <CoverImage {coverPath} label={title} />

  <div class="grid min-w-0 gap-3.5 px-1 pb-1">
    <div class="grid gap-2.5">
      <div class="flex flex-wrap items-center gap-2">
        <StatusBadge label={compatibility.badge} />
        <span class="rounded-full border border-slate-200/80 bg-slate-50/90 px-2.5 py-1 text-[0.68rem] font-semibold uppercase tracking-[0.18em] text-slate-500">
          {itemType}
        </span>
      </div>

      <div class="grid gap-1.5">
        <h3 class="line-clamp-2 text-[1.02rem] font-semibold leading-6 text-slate-950">{title}</h3>
        <p class="text-sm leading-6 text-slate-600">{compatibility.summaryCopy}</p>
      </div>
    </div>

    {#if assignedMonitorLabels.length > 0}
      <div class="grid gap-1.5 rounded-2xl border border-slate-200/80 bg-slate-50/80 px-3.5 py-3">
        <p class="text-[0.68rem] font-semibold uppercase tracking-[0.18em] text-slate-500">
          Assigned to
        </p>
        <p class="text-sm text-slate-700">{assignedMonitorLabels.join(' • ')}</p>
      </div>
    {/if}
  </div>
</Card>
