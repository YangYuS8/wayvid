<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  type DialogProps = HTMLAttributes<HTMLDivElement> & {
    children?: Snippet;
  };

  let { children, ...restProps }: DialogProps = $props();
</script>

<div data-slot="dialog" {...restProps}>
  {@render children?.()}
</div>
