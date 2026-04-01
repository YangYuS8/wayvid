<script lang="ts">
  import { Dialog as DialogPrimitive } from 'bits-ui';

  import { cn } from '$lib/ui/utils';

  let { class: className = '', children, ...restProps }: DialogPrimitive.DescriptionProps = $props();
</script>

<DialogPrimitive.Description
  class={cn('text-sm text-slate-600', className)}
  data-slot="dialog-description"
  {...restProps}
>
  {@render children?.()}
</DialogPrimitive.Description>
