<script lang="ts">
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { LibraryItemDetail } from '$lib/types';

  export let detail: LibraryItemDetail | null = null;
  export let loading = false;
  export let error: string | null = null;
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
          <StatusBadge label={detail.source} />
          <StatusBadge label={detail.itemType} />
        </div>

        {#if detail.description}
          <p>{detail.description}</p>
        {/if}

        {#if detail.tags.length > 0}
          <p class="tags">{detail.tags.join(' • ')}</p>
        {/if}
      </div>
    </div>
  {:else}
    <p>Select a Library item to inspect its current detail payload.</p>
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

  .tags {
    color: #5a6978;
  }
</style>
