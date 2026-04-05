<script lang="ts">
import { Card } from '$lib/ui/card';
  import CoverImage from '$lib/components/CoverImage.svelte';
import { copy } from '$lib/i18n';

  export let title: string;
export let coverPath: string | null = null;
export let selected = false;
export let assignedMonitorLabels: string[] = [];
export let selectLabel: string | null = null;
export let onSelect: (() => void) | undefined = undefined;
</script>

<Card
  class={`relative lwe-panel-compact group transition duration-150 hover:-translate-y-0.5 hover:border-border/90 hover:bg-accent/15 hover:shadow-[0_24px_56px_rgba(15,23,42,0.12)] ${selected ? 'border-primary/70 ring-1 ring-primary/20' : ''}`}
>
  {#if onSelect && selectLabel}
    <button
      type="button"
      class="absolute inset-0 z-10 rounded-[1.125rem] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
      aria-label={selectLabel}
      aria-pressed={selected}
      onclick={onSelect}
    ></button>
  {/if}

  <div class={`grid gap-4 ${onSelect ? 'pointer-events-none relative z-0' : ''}`}>
    <CoverImage {coverPath} label={title} square={true} />

    <div class="grid min-w-0 gap-2 px-1 pb-1">
      <h3 class="line-clamp-2 text-base font-semibold leading-6 text-foreground">{title}</h3>

      {#if assignedMonitorLabels.length > 0}
        <div class="lwe-subpanel gap-1.5 px-3.5 py-3">
          <p class="text-[0.68rem] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
            {$copy.components.itemCard.assignedTo}
          </p>
          <p class="text-sm text-foreground/85">{assignedMonitorLabels.join(' • ')}</p>
        </div>
      {/if}
    </div>
  </div>
</Card>
