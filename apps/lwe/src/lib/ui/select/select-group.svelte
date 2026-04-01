<script lang="ts">
  import { Select as SelectPrimitive } from 'bits-ui';

  let { children, ...restProps }: SelectPrimitive.GroupProps = $props();
</script>

<SelectPrimitive.Group data-slot="select-group" {...restProps}>
  {@render children?.()}
</SelectPrimitive.Group>
