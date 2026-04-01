<script lang="ts">
  import { Dialog as DialogPrimitive } from 'bits-ui';

  let { open = $bindable(false), children, ...restProps }: DialogPrimitive.RootProps = $props();
</script>

<DialogPrimitive.Root bind:open {...restProps}>
  {@render children?.()}
</DialogPrimitive.Root>
