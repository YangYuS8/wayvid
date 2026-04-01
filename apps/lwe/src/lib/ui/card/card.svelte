<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  import { cn } from '$lib/ui/utils';

  type CardProps = HTMLAttributes<HTMLDivElement> & {
    children?: Snippet;
  };

  let { class: className = '', children, ...restProps }: CardProps = $props();
</script>

<div
  class={cn('rounded-xl border border-slate-200 bg-white text-slate-950 shadow-sm', className)}
  data-slot="card"
  {...restProps}
>
  {@render children?.()}
</div>
