<script lang="ts">
  import { shouldRenderCoverImage } from '$lib/components/cover-image';

  export let coverPath: string | null = null;
  export let label = 'cover';

  let loadFailed = false;
  let previousCoverPath: string | null = null;

  $: if (coverPath !== previousCoverPath) {
    previousCoverPath = coverPath;
    loadFailed = false;
  }

  $: showCoverImage = shouldRenderCoverImage(coverPath, loadFailed);
</script>

{#if showCoverImage}
  <img class="cover-image" src={coverPath ?? undefined} alt={label} loading="lazy" on:error={() => (loadFailed = true)} />
{:else}
  <div class="cover-placeholder" aria-label={`${label} placeholder`}>
    <span>No Cover</span>
  </div>
{/if}

<style>
  .cover-image,
  .cover-placeholder {
    width: 100%;
    aspect-ratio: 16 / 9;
    border-radius: 14px;
    overflow: hidden;
  }

  .cover-image {
    display: block;
    object-fit: cover;
    background: #d8dde5;
  }

  .cover-placeholder {
    display: grid;
    place-items: center;
    background: linear-gradient(135deg, #eef2f6, #d7e1ea);
    color: #435260;
    font-size: 0.78rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
</style>
