<script lang="ts">
  import { Dialog as DialogPrimitive } from 'bits-ui';

  let { type = 'button', children, ...restProps }: DialogPrimitive.CloseProps = $props();
</script>

<DialogPrimitive.Close data-slot="dialog-close" {type} {...restProps}>
  {@render children?.()}
</DialogPrimitive.Close>
