<script lang="ts">
  import { Select as SelectPrimitive } from 'bits-ui';

  import { cn } from '$lib/ui/utils';

  const portalDisabled = typeof document === 'undefined';

  type SelectContentProps = SelectPrimitive.ContentProps & {
    viewportClass?: string;
  };

  let {
    class: className = '',
    viewportClass = '',
    children,
    forceMount = true,
    sideOffset = 8,
    ...restProps
  }: SelectContentProps = $props();
</script>

<SelectPrimitive.Portal disabled={portalDisabled}>
  <SelectPrimitive.Content
    class={cn(
      'z-50 max-h-96 min-w-[8rem] overflow-hidden rounded-md border border-border bg-popover text-popover-foreground shadow-lg',
      className
    )}
    data-slot="select-content"
    {forceMount}
    {sideOffset}
    {...restProps}
  >
    <SelectPrimitive.Viewport class={cn('p-1', viewportClass)}>
      {@render children?.()}
    </SelectPrimitive.Viewport>
  </SelectPrimitive.Content>
</SelectPrimitive.Portal>
