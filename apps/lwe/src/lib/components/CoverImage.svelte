<script lang="ts">
  export let coverPath: string | null = null;
  export let label = 'cover';
</script>

{#if coverPath}
  <img class="cover-image" src={coverPath} alt={label} loading="lazy" />
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
