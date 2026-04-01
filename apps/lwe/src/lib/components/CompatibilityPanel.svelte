<script lang="ts">
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { CompatibilityExplanationModel } from '$lib/types';

  export let compatibility: CompatibilityExplanationModel;
</script>

<Card
  class="grid gap-4 border-slate-200/80 bg-slate-50/80 p-4 shadow-none"
  aria-label="Compatibility explanation"
>
  <div class="grid gap-3 sm:grid-cols-[auto_minmax(0,1fr)] sm:items-start">
    <StatusBadge label={compatibility.badge} />

    <div class="grid gap-1.5">
      <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
        Compatibility
      </p>
      <h3 class="text-base font-semibold text-slate-950">{compatibility.headline}</h3>
      <p class="text-sm leading-6 text-slate-700">{compatibility.detail}</p>
    </div>
  </div>

  {#if compatibility.nextStepCopy}
    <Separator class="bg-slate-200/80" />
    <p class="text-sm text-slate-600">Next step: {compatibility.nextStepCopy}</p>
  {/if}
</Card>
