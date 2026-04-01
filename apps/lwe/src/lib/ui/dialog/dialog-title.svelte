<script lang="ts">
  import { Dialog as DialogPrimitive } from 'bits-ui';

  import { cn } from '$lib/ui/utils';

  let { class: className = '', children, ...restProps }: DialogPrimitive.TitleProps = $props();
</script>

<DialogPrimitive.Title
  class={cn('text-lg font-semibold text-slate-950', className)}
  data-slot="dialog-title"
  {...restProps}
>
  {@render children?.()}
</DialogPrimitive.Title>
