<script lang="ts">
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import { copy, getSceneIssueLabel, localizeSceneCompatibility } from '$lib/i18n';
  import type { CompatibilityExplanationModel } from '$lib/types';

export let compatibility: CompatibilityExplanationModel;
export let badgeLabel: string | null = null;
  export let badgeVariantKey: string | null = null;

  $: compatibilityCopy = $copy.components.compatibilityPanel;
  $: explanation = localizeSceneCompatibility($copy, compatibility);
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
      <h3 class="text-base font-semibold tracking-tight text-foreground">{explanation.headline}</h3>
      <p class="text-sm leading-6 text-foreground/85">{explanation.detail}</p>
    </div>
  </div>

  {#if compatibility.sceneAssessment?.issues.length}
    <div class="grid min-w-0 gap-2">
      <h4 class="text-sm font-semibold text-foreground">{compatibilityCopy.scene.blockers}</h4>
      <ul class="grid gap-3" aria-label={compatibilityCopy.scene.blockers}>
        {#each compatibility.sceneAssessment.issues as issue}
          <li class="lwe-wrap-safe rounded-xl border border-border/80 px-3.5 py-3">
            <p class="text-sm font-medium text-foreground">{getSceneIssueLabel($copy, issue.code)}</p>
            <p class="mt-1 text-sm leading-6 text-muted-foreground">{issue.detail}</p>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  {#if explanation.nextStepCopy}
    <Separator class="bg-border/80" />
    <p class="rounded-xl bg-card/70 px-3.5 py-3 text-sm leading-6 text-muted-foreground">
      {compatibilityCopy.nextStepPrefix}{explanation.nextStepCopy}
    </p>
  {/if}
</Card>
