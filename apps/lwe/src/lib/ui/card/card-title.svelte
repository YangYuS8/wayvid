<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type CardTitleProps = HTMLAttributes<HTMLHeadingElement> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: CardTitleProps = $props();
</script>

<h3 class={cn('font-semibold leading-none tracking-tight', className)} data-slot="card-title" {...restProps}>
  {@render children?.()}
</h3>
