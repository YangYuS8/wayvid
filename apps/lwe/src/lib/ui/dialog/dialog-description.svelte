<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type DialogDescriptionProps = HTMLAttributes<HTMLParagraphElement> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: DialogDescriptionProps = $props();
</script>

<p class={cn('text-sm text-slate-500', className)} data-slot="dialog-description" {...restProps}>
  {@render children?.()}
</p>
