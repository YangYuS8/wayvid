<script lang="ts">
  import { Button } from '$lib/ui/button';
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';

  export let displayName: string;
  export let monitorId: string;
  export let resolution: string | null = null;
  export let currentItemLabel: string;
  export let currentCoverPath: string | null = null;
  export let runtimeStatus: string | null = null;
  export let restoreState: string | null = null;
  export let restoreIssue: string | null = null;
  export let missing = false;
  let detailsExpanded = false;

  $: statusLabels = [runtimeStatus, restoreState].filter((value): value is string => Boolean(value));
  $: hasStateDetails = statusLabels.length > 0 || Boolean(restoreIssue);
  $: issueBannerClass = restoreIssue ? 'lwe-warning-banner' : 'lwe-info-banner';
</script>

<Card
  class="lwe-panel-compact"
  aria-label={`Desktop monitor ${displayName}`}
>
  <CoverImage coverPath={currentCoverPath} label={`${displayName} current item`} />

  <div class="grid gap-4 px-1 pb-1">
    <div class="grid gap-3.5">
      <div class="flex flex-wrap items-center gap-2">
        <StatusBadge label="desktop" />
        {#if missing}
          <StatusBadge label="missing_monitor" />
        {/if}

        {#each statusLabels as statusLabel}
          <StatusBadge label={statusLabel} />
        {/each}
      </div>

      <div class="grid gap-1.5">
        <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Monitor</p>
        <h3 class="lwe-heading-md lwe-wrap-safe">{displayName}</h3>
        <p class="lwe-wrap-safe text-sm text-slate-600">
          {monitorId}
          {#if resolution}
            <span class="text-slate-400"> • </span>{resolution}
          {/if}
        </p>
      </div>
    </div>

    <div class="lwe-subpanel">
      <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Current item</p>
      <p class="lwe-wrap-safe text-sm leading-6 text-slate-800">{currentItemLabel}</p>
    </div>

    {#if hasStateDetails}
      <Separator class="bg-slate-200/80" />
      <div class="lwe-subpanel gap-3">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Restore state</p>

          <Button
            variant="outline"
            size="sm"
            class="w-fit"
            aria-expanded={detailsExpanded}
            onclick={() => {
              detailsExpanded = !detailsExpanded;
            }}
          >
            {detailsExpanded ? 'Hide status details' : 'View status details'}
          </Button>
        </div>

        {#if statusLabels.length > 0}
          <div class="flex flex-wrap gap-2">
            {#each statusLabels as statusLabel}
              <StatusBadge label={statusLabel} />
            {/each}
          </div>
        {/if}

        {#if restoreIssue}
          <p class={issueBannerClass} role="status" aria-live="polite">
            {restoreIssue}
          </p>
        {/if}

        {#if detailsExpanded}
          <div class="grid gap-4 rounded-2xl border border-slate-200/80 bg-white/80 p-4">
            <Separator class="bg-slate-200/80" />

            <div class="grid gap-1.5">
              <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Monitor status</p>
              <h4 class="lwe-heading-md lwe-wrap-safe">{displayName}</h4>
              <p class="lwe-wrap-safe text-sm text-slate-600">{monitorId}</p>
            </div>

            {#if statusLabels.length > 0}
              <div class="flex flex-wrap gap-2">
                {#each statusLabels as statusLabel}
                  <StatusBadge label={statusLabel} />
                {/each}
              </div>
            {/if}

            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
              {restoreIssue ?? 'This monitor has state metadata available, but no additional restore issue was reported.'}
            </p>

            <p class="text-xs font-medium uppercase tracking-[0.16em] text-slate-500">
              Expand this section to review the latest restore status for this display.
            </p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</Card>
