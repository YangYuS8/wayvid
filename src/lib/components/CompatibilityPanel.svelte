<script lang="ts">
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import { copy } from '$lib/i18n';
  import type { CompatibilityExplanationModel } from '$lib/types';

export let compatibility: CompatibilityExplanationModel;
export let badgeLabel: string | null = null;
  export let badgeVariantKey: string | null = null;

  $: compatibilityCopy = $copy.components.compatibilityPanel;
</script>

<Card
  class="lwe-subpanel gap-4 shadow-none"
  aria-label={compatibilityCopy.ariaLabel}
>
  <div class="grid gap-3 sm:grid-cols-[auto_minmax(0,1fr)] sm:items-start">
    <StatusBadge label={badgeLabel ?? compatibility.badge} variantKey={badgeVariantKey ?? compatibility.badge} />

    <div class="grid gap-2">
      <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-muted-foreground">
        {compatibilityCopy.title}
      </p>
      <h3 class="text-base font-semibold tracking-tight text-foreground">{compatibility.headline}</h3>
      <p class="text-sm leading-6 text-foreground/85">{compatibility.detail}</p>
    </div>
  </div>

  {#if compatibility.nextStepCopy}
    <Separator class="bg-border/80" />
    <p class="rounded-xl bg-card/70 px-3.5 py-3 text-sm leading-6 text-muted-foreground">
      {compatibilityCopy.nextStepPrefix}{compatibility.nextStepCopy}
    </p>
  {/if}
</Card>
