<script lang="ts">
  import type { HTMLDialogAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type DialogProps = HTMLDialogAttributes & {
    open?: boolean;
  };

  let { class: className = '', children, open = false, ...restProps }: DialogProps = $props();
</script>

<dialog
  class={cn(
    'w-full max-w-lg rounded-xl border border-slate-200 bg-white p-6 text-slate-950 shadow-lg backdrop:bg-slate-950/40',
    className
  )}
  data-slot="dialog"
  {open}
  {...restProps}
>
  {@render children?.()}
</dialog>
