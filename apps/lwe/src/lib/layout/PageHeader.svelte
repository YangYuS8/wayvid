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

<header class="grid gap-4 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-start">
  <div class="grid gap-3">
    {#if eyebrow}
      <p class="m-0 text-[0.72rem] font-semibold uppercase tracking-[0.24em] text-slate-500">{eyebrow}</p>
    {/if}

    <div class="grid gap-2">
      <h1 class="m-0 text-[clamp(1.8rem,4vw,2.5rem)] font-semibold leading-none tracking-tight text-slate-950">
        {title}
      </h1>

      {#if subtitle}
        <p class="m-0 max-w-[64ch] text-sm leading-7 text-slate-600 sm:text-[0.95rem]">{subtitle}</p>
      {/if}
    </div>
  </div>

  {#if actions}
    <div class="grid justify-items-start gap-2 sm:justify-items-end sm:self-center">
      {@render actions()}
    </div>
  {/if}
</header>
