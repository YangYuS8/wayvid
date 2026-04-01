<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { cva, type VariantProps } from 'class-variance-authority';

  import { cn } from '$lib/ui/utils';

  export const buttonVariants = cva(
    'inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium transition-colors disabled:pointer-events-none disabled:opacity-50',
    {
      variants: {
        variant: {
          default: 'bg-slate-900 text-slate-50',
          secondary: 'bg-slate-100 text-slate-900',
          outline: 'border border-slate-200 bg-white text-slate-900',
          ghost: 'text-slate-900'
        },
        size: {
          default: 'h-9 px-4 py-2',
          sm: 'h-8 rounded-md px-3',
          lg: 'h-10 rounded-md px-8',
          icon: 'h-9 w-9'
        }
      },
      defaultVariants: {
        variant: 'default',
        size: 'default'
      }
    }
  );

  type ButtonProps = HTMLButtonAttributes &
    VariantProps<typeof buttonVariants> & {
      children?: Snippet;
    };

  let {
    class: className = '',
    variant = 'default',
    size = 'default',
    children,
    type = 'button',
    ...restProps
  }: ButtonProps = $props();
</script>

<button
  class={cn(buttonVariants({ variant, size }), className)}
  {type}
  data-slot="button"
  {...restProps}
>
  {@render children?.()}
</button>
