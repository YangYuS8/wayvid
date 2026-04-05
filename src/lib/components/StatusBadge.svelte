<script lang="ts">
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

  const positiveStatePatterns = [
    'supported',
    'running',
    'restored',
    'synced',
    'available',
    'applied',
    'ok',
    'ready'
  ];

  const neutralStatePatterns = [
    'workshop',
    'library',
    'video',
    'scene',
    'web',
    'application',
    'desktop',
    'monitor'
  ];

  const isPositiveState = (value: string) => {
    const normalized = value.toLowerCase();
    return positiveStatePatterns.some((pattern) => normalized.includes(pattern));
  };

  const isNeutralState = (value: string) => {
    const normalized = value.toLowerCase();
    return neutralStatePatterns.some((pattern) => normalized.includes(pattern));
  };

  $: badgeStateKey = (variantKey ?? label).toLowerCase();
  $: badgeClass = cn(
    'inline-flex items-center rounded-full border px-2.5 py-1 text-[0.68rem] font-semibold uppercase tracking-[0.18em] shadow-none transition-colors !text-foreground',
    badgeVariant === 'secondary' && isNeutralState(badgeStateKey) && 'border-indigo-300/75 bg-indigo-100/85 dark:border-indigo-400/40 dark:bg-indigo-500/25',
    badgeVariant === 'secondary' && isPositiveState(badgeStateKey) && 'border-emerald-300/75 bg-emerald-100/85 dark:border-emerald-400/45 dark:bg-emerald-500/22',
    badgeVariant === 'secondary' && !isNeutralState(badgeStateKey) && !isPositiveState(badgeStateKey) && 'border-primary/35 bg-primary/12',
    badgeVariant === 'outline' && 'border-amber-300/85 bg-amber-100/80 dark:border-amber-400/45 dark:bg-amber-500/22',
    badgeVariant === 'destructive' && 'border-destructive/50 bg-destructive/12'
  );
</script>

<span class={badgeClass} data-slot="badge">
  {badgeLabel}
</span>
