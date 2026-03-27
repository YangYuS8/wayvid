<script lang="ts">
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { WorkshopItemDetail } from '$lib/types';

  export let detail: WorkshopItemDetail | null = null;
  export let loading = false;
  export let error: string | null = null;
  export let openInSteam: (() => Promise<void>) | null = null;
</script>

<section class="panel">
  {#if loading}
    <p>Loading item details...</p>
  {:else if error}
    <p>{error}</p>
  {:else if detail}
    <div class="panel-body">
      <CoverImage coverPath={detail.coverPath} label={detail.title} />

      <div class="copy">
        <h2>{detail.title}</h2>

        <div class="badges">
          <StatusBadge label={detail.compatibilityBadge} />
          <StatusBadge label={detail.syncStatus} />
        </div>

        <p class="meta">{detail.itemType}</p>

        {#if detail.compatibilityNote}
          <p>{detail.compatibilityNote}</p>
        {/if}

        {#if detail.description}
          <p>{detail.description}</p>
        {/if}

        {#if detail.tags.length > 0}
          <p class="tags">{detail.tags.join(' • ')}</p>
        {/if}

        <button type="button" on:click={() => openInSteam?.()} disabled={!openInSteam}>
          Open In Steam
        </button>
      </div>
    </div>
  {:else}
    <p>Select a Workshop item to inspect its current detail payload.</p>
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
  .badges {
    display: grid;
    gap: 0.9rem;
  }

  .badges {
    grid-template-columns: repeat(auto-fit, minmax(0, max-content));
  }

  h2,
  p {
    margin: 0;
  }

  .meta,
  .tags {
    color: #5a6978;
    text-transform: capitalize;
  }

  button {
    width: fit-content;
    border: 0;
    border-radius: 999px;
    padding: 0.7rem 1rem;
    background: #23456e;
    color: #fff;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
