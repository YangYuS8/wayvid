<script lang="ts">
  import { Select as SelectPrimitive } from 'bits-ui';

  import { cn } from '$lib/ui/utils';

  let { class: className = '', children, ...restProps }: SelectPrimitive.TriggerProps = $props();
</script>

<SelectPrimitive.Trigger
  class={cn(
    'flex h-9 w-full items-center justify-between rounded-md border border-slate-200 bg-white px-3 py-2 text-sm text-slate-950 shadow-sm outline-none transition-colors hover:border-slate-300 focus-visible:ring-2 focus-visible:ring-slate-400 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
    className
  )}
  data-slot="select-trigger"
  {...restProps}
>
  {@render children?.()}
</SelectPrimitive.Trigger>
