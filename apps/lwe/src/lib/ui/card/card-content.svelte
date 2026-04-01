<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type CardContentProps = HTMLAttributes<HTMLDivElement> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: CardContentProps = $props();
</script>

<div class={cn('p-6 pt-0', className)} data-slot="card-content" {...restProps}>
  {@render children?.()}
</div>
