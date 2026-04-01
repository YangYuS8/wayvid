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
          default: 'border-transparent bg-slate-900 text-slate-50',
          secondary: 'border-transparent bg-slate-100 text-slate-900',
          outline: 'border-slate-200 text-slate-900',
          destructive: 'border-transparent bg-red-600 text-white'
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
