<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type CardFooterProps = HTMLAttributes<HTMLDivElement> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: CardFooterProps = $props();
</script>

<div class={cn('flex items-center p-6 pt-0', className)} data-slot="card-footer" {...restProps}>
  {@render children?.()}
</div>
