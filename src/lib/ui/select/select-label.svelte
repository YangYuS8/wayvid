<script lang="ts">
  import { Select as SelectPrimitive } from 'bits-ui';

  import { cn } from '$lib/ui/utils';

  let { class: className = '', children, ...restProps }: SelectPrimitive.GroupHeadingProps = $props();
</script>

<SelectPrimitive.GroupHeading
  class={cn('px-2 py-1.5 text-xs font-semibold uppercase tracking-[0.12em] text-muted-foreground', className)}
  data-slot="select-label"
  {...restProps}
>
  {@render children?.()}
</SelectPrimitive.GroupHeading>
