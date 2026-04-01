<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type DialogHeaderProps = HTMLAttributes<HTMLDivElement> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: DialogHeaderProps = $props();
</script>

<div class={cn('flex flex-col space-y-2 p-6', className)} data-slot="dialog-header" {...restProps}>
  {@render children?.()}
</div>
