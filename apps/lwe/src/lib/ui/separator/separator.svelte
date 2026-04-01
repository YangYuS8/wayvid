<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type SeparatorProps = HTMLAttributes<HTMLDivElement> & {
    decorative?: boolean;
    orientation?: 'horizontal' | 'vertical';
  };

  let {
    class: className = '',
    decorative = true,
    orientation = 'horizontal',
    ...restProps
  }: SeparatorProps = $props();
</script>

<div
  aria-orientation={orientation}
  class={cn(
    'shrink-0 bg-border',
    orientation === 'horizontal' ? 'h-px w-full' : 'h-full w-px',
    className
  )}
  data-slot="separator"
  role={decorative ? 'none' : 'separator'}
  {...restProps}
></div>
