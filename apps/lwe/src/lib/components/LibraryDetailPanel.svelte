<script lang="ts">
  import { Card } from '$lib/ui/card';
  import { Separator } from '$lib/ui/separator';
  import { Button } from '$lib/ui/button';
  import * as Select from '$lib/ui/select';
  import CompatibilityPanel from '$lib/components/CompatibilityPanel.svelte';
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { DesktopMonitorSummary, LibraryItemDetail, LibraryPageSnapshot } from '$lib/types';
  import { resolveLibraryAvailabilityIssues } from '../../routes/library/page-state';

  export let detail: LibraryItemDetail | null = null;
  export let snapshot: LibraryPageSnapshot | null = null;
  export let loading = false;
  export let error: string | null = null;
  export let monitors: DesktopMonitorSummary[] = [];
  export let selectedMonitorId = '';
  export let applyDisabled = true;
  export let applying = false;
  export let applyMessage: string | null = null;
  export let onApply: (() => void) | undefined = undefined;
  export let onMonitorChange: ((monitorId: string) => void) | undefined = undefined;

  $: availabilitySource = detail ?? snapshot;
  $: issueMessages = availabilitySource ? resolveLibraryAvailabilityIssues(availabilitySource) : [];
  $: assignedMonitorLabels = detail?.assignedMonitorLabels ?? [];
</script>

<Card class="lwe-panel">
  {#if loading}
    <div class="lwe-subpanel gap-3" role="status" aria-live="polite">
      <p class="lwe-eyebrow">Library detail</p>
      <p class="text-sm leading-6 text-slate-600">Loading item details…</p>
    </div>
  {:else if error}
    <div class="lwe-subpanel gap-3">
      <p class="lwe-eyebrow">Library detail</p>
      <p class="lwe-warning-banner lwe-wrap-safe" role="alert" aria-live="assertive">
        {error}
      </p>
    </div>
  {:else if detail}
    <div class="grid gap-6 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)] lg:items-start">
      <CoverImage coverPath={detail.coverPath} label={detail.title} />

      <div class="grid min-w-0 gap-5">
        <div class="grid gap-3.5">
          <div class="grid gap-2">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              Library item
            </p>
            <h2 class="lwe-heading-lg lwe-wrap-safe">{detail.title}</h2>
          </div>

          <div class="flex flex-wrap gap-2">
            <StatusBadge label={detail.compatibility.badge} />
            <StatusBadge label={detail.source} />
            <StatusBadge label={detail.itemType} />
          </div>
        </div>

        {#if issueMessages.length}
          <div class="grid gap-2.5" aria-live="polite">
            {#each issueMessages as issue}
              <p class="lwe-info-banner lwe-wrap-safe">
                {issue}
              </p>
            {/each}
          </div>
        {/if}

        {#if assignedMonitorLabels.length > 0}
          <div class="lwe-subpanel" aria-live="polite">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              Assigned monitors
            </p>
            <p class="lwe-wrap-safe text-sm text-slate-700">{assignedMonitorLabels.join(' • ')}</p>
          </div>
        {/if}

        <div class="lwe-subpanel gap-3.5" aria-label="Apply this item to a monitor">
          <div class="grid gap-1.5">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Actions</p>
            <h3 class="text-base font-semibold tracking-tight text-slate-950">Apply</h3>
            <p class="text-sm leading-6 text-slate-600">
              Choose a monitor, then apply this item without leaving Library.
            </p>
          </div>

          <div class="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end">
            <label class="grid gap-1.5">
              <span class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
                Monitor
              </span>

              <Select.Root
                type="single"
                name="libraryMonitor"
                value={selectedMonitorId}
                onValueChange={(value) => onMonitorChange?.(value)}
                disabled={monitors.length === 0}
              >
                <Select.Trigger aria-label="Apply target monitor">
                  {selectedMonitorId
                    ? monitors.find((monitor) => monitor.monitorId === selectedMonitorId)?.displayName ?? selectedMonitorId
                    : monitors.length > 0
                      ? 'Select a monitor'
                      : 'No monitors available'}
                </Select.Trigger>

                <Select.Content>
                  {#each monitors as monitor}
                    <Select.Item value={monitor.monitorId} label={monitor.displayName}>
                      {monitor.displayName}
                    </Select.Item>
                  {/each}
                </Select.Content>
              </Select.Root>
            </label>

            <Button onclick={onApply} disabled={applyDisabled || applying}>
              {applying ? 'Applying…' : 'Apply'}
            </Button>
          </div>

          {#if applyMessage}
            <p class="lwe-info-banner" role="status" aria-live="polite">{applyMessage}</p>
          {/if}
        </div>

        <CompatibilityPanel compatibility={detail.compatibility} />

        <Separator class="bg-slate-200/80" />

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">
              Description
            </p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
              {detail.description ?? 'No description available for this item yet.'}
            </p>
          </div>

          <div class="lwe-subpanel">
            <p class="text-[0.7rem] font-semibold uppercase tracking-[0.2em] text-slate-500">Tags</p>
            <p class="lwe-wrap-safe text-sm leading-6 text-slate-700">
              {detail.tags.length > 0 ? detail.tags.join(' • ') : 'No tags are attached to this item.'}
            </p>
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div class="grid gap-4">
      {#if issueMessages.length}
        <div class="grid gap-2.5" aria-live="polite">
          {#each issueMessages as issue}
            <p class="lwe-info-banner lwe-wrap-safe">
              {issue}
            </p>
          {/each}
        </div>
      {/if}

      <div class="lwe-subpanel gap-3 border-dashed" role="status" aria-live="polite">
        <p class="lwe-eyebrow">Library detail</p>
        <p class="text-sm leading-6 text-slate-600">Select a Library item to inspect its current detail payload.</p>
      </div>
    </div>
  {/if}
</Card>
