<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Select as SelectPrimitive } from 'bits-ui';

  import { cn } from '$lib/ui/utils';

  type SelectItemProps = Omit<SelectPrimitive.ItemProps, 'children'> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: SelectItemProps = $props();
</script>

<SelectPrimitive.Item
  class={cn(
    'relative flex w-full cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors data-[disabled]:pointer-events-none data-[disabled]:opacity-50 data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground',
    className
  )}
  data-slot="select-item"
  {...restProps}
>
  {@render children?.()}
</SelectPrimitive.Item>
