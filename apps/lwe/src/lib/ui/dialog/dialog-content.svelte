<script lang="ts">
  import type { HTMLDialogAttributes } from 'svelte/elements';
  import type { Snippet } from 'svelte';

  import { cn } from '$lib/ui/utils';

  type DialogContentProps = HTMLDialogAttributes & {
    children?: Snippet;
  };

  let { class: className = '', children, open = true, ...restProps }: DialogContentProps = $props();
</script>

<dialog
  aria-modal="true"
  class={cn('w-full max-w-lg rounded-xl border border-slate-200 bg-white p-0 text-slate-950 shadow-lg', className)}
  data-slot="dialog-content"
  {open}
  {...restProps}
>
  {@render children?.()}
</dialog>
