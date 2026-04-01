<script lang="ts">
  import { Dialog as DialogPrimitive } from 'bits-ui';

  import { cn } from '$lib/ui/utils';

  const portalDisabled = typeof document === 'undefined';

  type DialogContentProps = DialogPrimitive.ContentProps & {
    overlayClass?: string;
  };

  let {
    class: className = '',
    overlayClass = '',
    children,
    ...restProps
  }: DialogContentProps = $props();
</script>

<DialogPrimitive.Portal disabled={portalDisabled}>
  <DialogPrimitive.Overlay
    class={cn('fixed inset-0 z-50 bg-foreground/40', overlayClass)}
    data-slot="dialog-overlay"
  />

  <DialogPrimitive.Content
    class={cn(
      'fixed left-1/2 top-1/2 z-50 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-xl border bg-card p-6 text-card-foreground shadow-lg',
      className
    )}
    data-slot="dialog-content"
    {...restProps}
  >
    {@render children?.()}
  </DialogPrimitive.Content>
</DialogPrimitive.Portal>
