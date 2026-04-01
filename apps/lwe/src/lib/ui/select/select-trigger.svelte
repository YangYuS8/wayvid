<script lang="ts">
  import { Select as SelectPrimitive } from 'bits-ui';

  import { cn } from '$lib/ui/utils';

  let { class: className = '', children, ...restProps }: SelectPrimitive.TriggerProps = $props();
</script>

<SelectPrimitive.Trigger
  class={cn(
    'flex h-9 w-full items-center justify-between rounded-md border bg-background px-3 py-2 text-sm text-foreground shadow-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50',
    className
  )}
  data-slot="select-trigger"
  {...restProps}
>
  {@render children?.()}
</SelectPrimitive.Trigger>
