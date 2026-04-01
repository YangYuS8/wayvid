<script lang="ts">
  import { Dialog as DialogPrimitive } from 'bits-ui';
  import { cn } from '$lib/ui/utils';

  let { class: className = '', type = 'button', children, ...restProps }: DialogPrimitive.CloseProps = $props();
</script>

<DialogPrimitive.Close
  class={cn(
    'inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50',
    className
  )}
  data-slot="dialog-close"
  {type}
  {...restProps}
>
  {@render children?.()}
</DialogPrimitive.Close>
