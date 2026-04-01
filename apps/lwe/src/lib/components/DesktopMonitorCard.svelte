<script lang="ts">
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
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

  $: statusLabels = [runtimeStatus, restoreState].filter((value): value is string => Boolean(value));
</script>

<Card
  class="grid gap-4 border-slate-200/80 bg-white/95 p-4 shadow-[0_18px_40px_rgba(15,23,42,0.08)]"
  aria-label={`Desktop monitor ${displayName}`}
>
  <div class="grid gap-3">
    <div class="flex flex-wrap items-center gap-2">
      {#if missing}
        <StatusBadge label="missing_monitor" />
      {/if}

      {#each statusLabels as statusLabel}
        <StatusBadge label={statusLabel} />
      {/each}
    </div>

    <div class="grid gap-1">
      <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Monitor</p>
      <h3 class="text-lg font-semibold text-slate-950">{displayName}</h3>
      <p class="text-sm text-slate-600">
        {monitorId}
        {#if resolution}
          <span class="text-slate-400"> • </span>{resolution}
        {/if}
      </p>
    </div>
  </div>

  {#if currentCoverPath}
    <div class="grid aspect-[16/9] place-items-center rounded-2xl border border-slate-200/80 bg-gradient-to-br from-slate-100 via-slate-50 to-slate-200 text-[0.68rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
      Cover available
    </div>
  {/if}

  <div class="grid gap-2 rounded-2xl border border-slate-200/80 bg-slate-50/80 p-4">
    <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Current item</p>
    <p class="text-sm leading-6 text-slate-800">{currentItemLabel}</p>
  </div>

  {#if restoreIssue}
    <Separator class="bg-slate-200/80" />
    <p class="rounded-xl border border-sky-200 bg-sky-50 px-4 py-3 text-sm text-sky-900">
      {restoreIssue}
    </p>
  {/if}
</Card>
