<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type DialogTriggerProps = HTMLButtonAttributes & {
    children?: Snippet;
  };

  let { class: className = '', children, type = 'button', ...restProps }: DialogTriggerProps = $props();
</script>

<button
  class={cn('inline-flex items-center justify-center rounded-md text-sm font-medium', className)}
  {type}
  data-slot="dialog-trigger"
  {...restProps}
>
  {@render children?.()}
</button>
