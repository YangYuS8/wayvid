<script lang="ts">
  import { Badge } from '$lib/ui/badge';
  import { cn } from '$lib/ui/utils';

  export let label: string;
  export let variantKey: string | null = null;

  const formatLabel = (value: string) =>
    value
      .split(/[_\s-]+/)
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ');

  const resolveBadgeVariant = (value: string): 'secondary' | 'outline' | 'destructive' => {
    const normalizedValue = value.toLowerCase();

    if (
      normalizedValue.includes('unsupported') ||
      normalizedValue.includes('missing') ||
      normalizedValue.includes('error')
    ) {
      return 'destructive';
    }

    if (normalizedValue.includes('partial') || normalizedValue.includes('unavailable')) {
      return 'outline';
    }

    return 'secondary';
  };

  $: badgeLabel = formatLabel(label);
  $: badgeVariant = resolveBadgeVariant(variantKey ?? label);
  $: badgeClass = cn(
    'px-2.5 py-1 text-[0.68rem] font-semibold uppercase tracking-[0.18em] shadow-none',
    badgeVariant === 'secondary' && 'border-sky-200/80 bg-sky-50 text-sky-900',
    badgeVariant === 'outline' && 'border-slate-300 bg-white/80 text-slate-700',
    badgeVariant === 'destructive' && 'border-red-200 bg-red-50 text-red-900'
  );
</script>

<Badge variant={badgeVariant} class={badgeClass}>
  {badgeLabel}
</Badge>
