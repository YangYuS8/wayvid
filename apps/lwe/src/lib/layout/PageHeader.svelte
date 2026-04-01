<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    eyebrow = null,
    title,
    subtitle = null,
    actions
  }: {
    eyebrow?: string | null;
    title: string;
    subtitle?: string | null;
    actions?: Snippet;
  } = $props();
</script>

<header class="page-header">
  <div class="copy">
    {#if eyebrow}
      <p class="eyebrow">{eyebrow}</p>
    {/if}

    <div class="title-block">
      <h1>{title}</h1>

      {#if subtitle}
        <p class="subtitle">{subtitle}</p>
      {/if}
    </div>
  </div>

  {#if actions}
    <div class="actions">
      {@render actions()}
    </div>
  {/if}
</header>

<style>
  .page-header,
  .copy,
  .title-block,
  .actions {
    display: grid;
    gap: 0.5rem;
  }

  .page-header {
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: start;
    gap: 1rem;
  }

  .eyebrow,
  h1,
  .subtitle {
    margin: 0;
  }

  .eyebrow {
    font-size: 0.77rem;
    font-weight: 600;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: #4a6381;
  }

  h1 {
    font-size: clamp(1.8rem, 4vw, 2.5rem);
    line-height: 1.05;
  }

  .subtitle {
    max-width: 64ch;
    color: #526272;
    line-height: 1.55;
  }

  .actions {
    justify-items: end;
    align-self: center;
  }

  @media (max-width: 720px) {
    .page-header {
      grid-template-columns: 1fr;
    }

    .actions {
      justify-items: start;
    }
  }
</style>
