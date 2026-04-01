<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type DialogTitleProps = HTMLAttributes<HTMLHeadingElement> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: DialogTitleProps = $props();
</script>

<h2 class={cn('text-lg font-semibold leading-none tracking-tight', className)} data-slot="dialog-title" {...restProps}>
  {@render children?.()}
</h2>
