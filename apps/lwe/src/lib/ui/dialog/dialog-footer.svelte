<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type DialogFooterProps = HTMLAttributes<HTMLDivElement> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: DialogFooterProps = $props();
</script>

<div class={cn('flex items-center justify-end gap-2 border-t border-slate-100 p-6', className)} data-slot="dialog-footer" {...restProps}>
  {@render children?.()}
</div>
