<script lang="ts">
  import type { HTMLOptionAttributes } from 'svelte/elements';
  import type { Snippet } from 'svelte';

  type SelectItemProps = HTMLOptionAttributes & {
    children?: Snippet;
  };

  let { children, ...restProps }: SelectItemProps = $props();
</script>

<option data-slot="select-item" {...restProps}>
  {@render children?.()}
</option>
