<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  let { class: className = '', children, ...restProps }: HTMLAttributes<HTMLDivElement> = $props();
</script>

<div class={cn('flex flex-col-reverse gap-2 pt-2 sm:flex-row sm:justify-end', className)} data-slot="dialog-footer" {...restProps}>
  {@render children?.()}
</div>
