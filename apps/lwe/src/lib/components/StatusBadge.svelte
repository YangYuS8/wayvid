<script lang="ts">
  import { Badge } from '$lib/ui/badge';

  export let label: string;

  const formatLabel = (value: string) =>
    value
      .split('_')
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ');

  const resolveBadgeVariant = (value: string): 'secondary' | 'outline' | 'destructive' => {
    if (value.includes('unsupported') || value.includes('missing') || value.includes('error')) {
      return 'destructive';
    }

    if (value.includes('partial') || value.includes('unavailable')) {
      return 'outline';
    }

    return 'secondary';
  };

  $: badgeLabel = formatLabel(label);
  $: badgeVariant = resolveBadgeVariant(label);
</script>

<Badge variant={badgeVariant} class="px-2.5 py-1 text-[0.68rem] font-semibold uppercase tracking-[0.18em]">
  {badgeLabel}
</Badge>
