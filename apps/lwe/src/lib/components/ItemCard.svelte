<script lang="ts">
  import CoverImage from '$lib/components/CoverImage.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';

  export let title: string;
  export let itemType: string;
  export let coverPath: string | null = null;
  export let primaryBadge: string;
  export let selected = false;
</script>

<div class:selected>
  <CoverImage {coverPath} label={title} />

  <div class="copy">
    <h3>{title}</h3>
    <p>{itemType}</p>
    <StatusBadge label={primaryBadge} />
  </div>
</div>

<style>
  div {
    display: grid;
    gap: 0.8rem;
    border: 1px solid rgba(33, 52, 72, 0.12);
    border-radius: 18px;
    padding: 0.9rem;
    background: #fff;
    box-shadow: 0 12px 28px rgba(25, 42, 62, 0.06);
    transition: border-color 0.18s ease, transform 0.18s ease;
  }

  div.selected {
    border-color: #23456e;
    transform: translateY(-1px);
  }

  .copy {
    display: grid;
    gap: 0.35rem;
    min-width: 0;
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
</style>
