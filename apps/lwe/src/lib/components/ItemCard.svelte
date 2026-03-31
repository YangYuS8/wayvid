<script lang="ts">
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

<div class="item-card" class:selected>
  <CoverImage {coverPath} label={title} />

  <div class="copy">
    <h3>{title}</h3>
    <p>{itemType}</p>
    <div class="summary">
      <StatusBadge label={compatibility.badge} />
      <p class="summary-copy">{compatibility.summaryCopy}</p>
    </div>

    {#if assignedMonitorLabels.length > 0}
      <div class="assignment-status">
        <p class="assignment-label">Assigned to</p>
        <p class="assignment-copy">{assignedMonitorLabels.join(' • ')}</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .item-card {
    display: grid;
    gap: 0.8rem;
    border: 1px solid rgba(33, 52, 72, 0.12);
    border-radius: 18px;
    padding: 0.9rem;
    background: #fff;
    box-shadow: 0 12px 28px rgba(25, 42, 62, 0.06);
    transition: border-color 0.18s ease, transform 0.18s ease;
  }

  .item-card.selected {
    border-color: #23456e;
    transform: translateY(-1px);
  }

  .copy {
    display: grid;
    gap: 0.35rem;
    min-width: 0;
  }

  .summary {
    display: grid;
    gap: 0.3rem;
  }

   .assignment-status {
    display: grid;
    gap: 0.2rem;
  }

  h3,
  p {
    margin: 0;
    overflow-wrap: anywhere;
  }

  h3 {
    font-size: 1rem;
    color: #162432;
    line-clamp: 2;
    -webkit-line-clamp: 2;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  p {
    color: #5a6978;
    text-transform: capitalize;
  }

  .summary-copy {
    font-size: 0.8rem;
    text-transform: none;
  }

  .assignment-label,
  .assignment-copy {
    font-size: 0.8rem;
    text-transform: none;
  }

  .assignment-label {
    color: #23456e;
    font-weight: 600;
  }
</style>
