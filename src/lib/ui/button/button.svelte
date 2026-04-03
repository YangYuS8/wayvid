<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { cva, type VariantProps } from 'class-variance-authority';

  import { cn } from '$lib/ui/utils';

export const buttonVariants = cva(
    'inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50',
    {
      variants: {
        variant: {
          default: 'bg-primary text-primary-foreground hover:opacity-90',
          secondary: 'bg-secondary text-secondary-foreground hover:opacity-90',
          outline: 'border bg-background text-foreground hover:bg-accent hover:text-accent-foreground',
          ghost: 'text-foreground hover:bg-accent hover:text-accent-foreground'
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
