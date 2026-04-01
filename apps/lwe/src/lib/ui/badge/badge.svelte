<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cva, type VariantProps } from 'class-variance-authority';

  import { cn } from '$lib/ui/utils';

  const badgeVariants = cva(
    'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors',
    {
      variants: {
        variant: {
          default: 'border-transparent bg-primary text-primary-foreground',
          secondary: 'border-transparent bg-secondary text-secondary-foreground',
          outline: 'border-border bg-background text-foreground',
          destructive: 'border-transparent bg-destructive text-destructive-foreground'
        }
      },
      defaultVariants: {
        variant: 'default'
      }
    }
  );

  type BadgeProps = HTMLAttributes<HTMLSpanElement> &
    VariantProps<typeof badgeVariants> & {
      children?: Snippet;
    };

  let { class: className = '', variant = 'default', children, ...restProps }: BadgeProps = $props();
</script>

<span class={cn(badgeVariants({ variant }), className)} data-slot="badge" {...restProps}>
  {@render children?.()}
</span>
