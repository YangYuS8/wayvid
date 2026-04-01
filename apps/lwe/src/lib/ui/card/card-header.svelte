<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type CardHeaderProps = HTMLAttributes<HTMLDivElement> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: CardHeaderProps = $props();
</script>

<div class={cn('flex flex-col space-y-1.5 p-6', className)} data-slot="card-header" {...restProps}>
  {@render children?.()}
</div>
