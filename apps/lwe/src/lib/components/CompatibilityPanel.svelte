<script lang="ts">
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import { copy } from '$lib/i18n';
  import type { CompatibilityExplanationModel } from '$lib/types';

  export let compatibility: CompatibilityExplanationModel;
  export let badgeLabel: string | null = null;

  $: compatibilityCopy = $copy.components.compatibilityPanel;
</script>

<Card
  class="lwe-subpanel gap-4 shadow-none"
  aria-label={compatibilityCopy.ariaLabel}
>
  <div class="grid gap-3 sm:grid-cols-[auto_minmax(0,1fr)] sm:items-start">
    <StatusBadge label={badgeLabel ?? compatibility.badge} />

    <div class="grid gap-2">
      <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
        {compatibilityCopy.title}
      </p>
      <h3 class="text-base font-semibold tracking-tight text-slate-950">{compatibility.headline}</h3>
      <p class="text-sm leading-6 text-slate-700">{compatibility.detail}</p>
    </div>
  </div>

  {#if compatibility.nextStepCopy}
    <Separator class="bg-slate-200/80" />
    <p class="rounded-xl bg-white/70 px-3.5 py-3 text-sm leading-6 text-slate-600">
      {compatibilityCopy.nextStepPrefix}{compatibility.nextStepCopy}
    </p>
  {/if}
</Card>
