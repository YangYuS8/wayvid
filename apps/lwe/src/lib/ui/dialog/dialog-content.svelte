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
    class={cn('fixed inset-0 z-50 bg-slate-950/40 data-[state=closed]:animate-out data-[state=open]:animate-in', overlayClass)}
    data-slot="dialog-overlay"
  />

  <DialogPrimitive.Content
    class={cn(
      'fixed left-1/2 top-1/2 z-50 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-xl border border-slate-200 bg-white p-6 text-slate-950 shadow-lg',
      className
    )}
    data-slot="dialog-content"
    {...restProps}
  >
    {@render children?.()}
  </DialogPrimitive.Content>
</DialogPrimitive.Portal>
