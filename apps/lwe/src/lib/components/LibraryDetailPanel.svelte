<script lang="ts">
  import CompatibilityPanel from '$lib/components/CompatibilityPanel.svelte';
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { LibraryItemDetail, LibraryPageSnapshot } from '$lib/types';
  import { resolveLibraryAvailabilityIssues } from '../../routes/library/page-state';

  export let detail: LibraryItemDetail | null = null;
  export let snapshot: LibraryPageSnapshot | null = null;
  export let loading = false;
  export let error: string | null = null;

  $: availabilitySource = detail ?? snapshot;
  $: issueMessages = availabilitySource ? resolveLibraryAvailabilityIssues(availabilitySource) : [];
  $: assignedMonitorLabels = detail?.assignedMonitorLabels ?? [];
</script>

<section class="panel">
  {#if loading}
    <p role="status" aria-live="polite">Loading item details…</p>
  {:else if error}
    <p role="alert" aria-live="assertive">{error}</p>
  {:else if detail}
    <div class="panel-body">
      <CoverImage coverPath={detail.coverPath} label={detail.title} />

      <div class="copy">
        <h2>{detail.title}</h2>
        <div class="badges">
          <StatusBadge label={detail.compatibility.badge} />
          <StatusBadge label={detail.source} />
          <StatusBadge label={detail.itemType} />
        </div>

        {#if issueMessages.length}
          <div class="issues" aria-live="polite">
            {#each issueMessages as issue}
              <p class="message warning">{issue}</p>
            {/each}
          </div>
        {/if}

        {#if assignedMonitorLabels.length > 0}
          <div class="assignments" aria-live="polite">
            <p class="assignment-label">Assigned monitors</p>
            <p>{assignedMonitorLabels.join(' • ')}</p>
          </div>
        {/if}

        <CompatibilityPanel compatibility={detail.compatibility} />

        {#if detail.description}
          <p>{detail.description}</p>
        {/if}

        {#if detail.tags.length > 0}
          <p class="tags">{detail.tags.join(' • ')}</p>
        {/if}
      </div>
    </div>
  {:else}
    <div class="copy">
      {#if issueMessages.length}
        <div class="issues" aria-live="polite">
          {#each issueMessages as issue}
            <p class="message warning">{issue}</p>
          {/each}
        </div>
      {/if}

      <p>Select a Library item to inspect its current detail payload.</p>
    </div>
  {/if}
</section>

<style>
  .panel {
    border: 1px solid rgba(33, 52, 72, 0.12);
    border-radius: 22px;
    padding: 1.1rem;
    background: rgba(255, 255, 255, 0.92);
    box-shadow: 0 16px 36px rgba(25, 42, 62, 0.08);
  }

  .panel-body,
  .copy,
  .badges,
   .issues,
   .assignments {
    display: grid;
    gap: 0.9rem;
  }

  .copy {
    min-width: 0;
  }

  .badges {
    grid-template-columns: repeat(auto-fit, minmax(0, max-content));
  }

  h2,
  p {
    margin: 0;
    overflow-wrap: anywhere;
  }

  .tags {
    color: #5a6978;
  }

  .message.warning {
    margin: 0;
    padding: 0.85rem 1rem;
    border-radius: 14px;
    background: rgba(15, 95, 154, 0.12);
  }

  .assignment-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: #23456e;
  }
</style>
