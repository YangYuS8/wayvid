<script lang="ts">
  import type { HTMLSelectAttributes } from 'svelte/elements';
  import type { Snippet } from 'svelte';

  import { cn } from '$lib/ui/utils';

  type SelectProps = HTMLSelectAttributes & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: SelectProps = $props();
</script>

<select
  class={cn(
    'flex h-9 w-full rounded-md border border-slate-200 bg-white px-3 py-2 text-sm text-slate-950 shadow-sm',
    className
  )}
  data-slot="select"
  {...restProps}
>
  {@render children?.()}
</select>
