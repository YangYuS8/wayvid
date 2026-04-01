<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import type { Snippet } from 'svelte';

  import { cn } from '$lib/ui/utils';

  type DialogCloseProps = HTMLButtonAttributes & {
    children?: Snippet;
  };

  let { class: className = '', children, type = 'button', ...restProps }: DialogCloseProps = $props();
</script>

<button
  class={cn('inline-flex items-center justify-center rounded-md text-sm font-medium', className)}
  {type}
  data-slot="dialog-close"
  {...restProps}
>
  {@render children?.()}
</button>
