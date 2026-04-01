<script lang="ts">
  import { Dialog as DialogPrimitive } from 'bits-ui';
  import { cn } from '$lib/ui/utils';

  let { class: className = '', children, ...restProps }: DialogPrimitive.TriggerProps = $props();
</script>

<DialogPrimitive.Trigger class={cn(className)} data-slot="dialog-trigger" {...restProps}>
  {@render children?.()}
</DialogPrimitive.Trigger>
