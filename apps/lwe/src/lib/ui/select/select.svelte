<script lang="ts">
  import type { HTMLSelectAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type SelectProps = HTMLSelectAttributes;

  let { class: className = '', children, ...restProps }: SelectProps = $props();
</script>

<select
  class={cn(
    'flex h-9 w-full rounded-md border border-slate-200 bg-white px-3 py-2 text-sm text-slate-950 shadow-sm outline-none transition-colors hover:border-slate-300 focus-visible:ring-2 focus-visible:ring-slate-400 focus-visible:ring-offset-2',
    className
  )}
  data-slot="select"
  {...restProps}
>
  {@render children?.()}
</select>
