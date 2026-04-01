<script lang="ts">
  import { Select as SelectPrimitive } from 'bits-ui';

  let { value = $bindable(), open = $bindable(false), children, ...restProps }: SelectPrimitive.RootProps =
    $props();
</script>

<SelectPrimitive.Root bind:value={value as never} bind:open {...restProps}>
  {@render children?.()}
</SelectPrimitive.Root>
