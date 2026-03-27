<script lang="ts">
  export let label: string;

  const formatLabel = (value: string) =>
    value
      .split('_')
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ');

  $: badgeLabel = formatLabel(label);
  $: badgeTone = label.includes('unsupported') || label.includes('missing') ? 'warn' : 'ok';
</script>

<span class={`status-badge ${badgeTone}`}>{badgeLabel}</span>

<style>
  .status-badge {
    display: inline-flex;
    align-items: center;
    border-radius: 999px;
    padding: 0.28rem 0.65rem;
    font-size: 0.74rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    background: rgba(35, 69, 110, 0.08);
    color: #23456e;
  }

  .status-badge.warn {
    background: rgba(160, 98, 23, 0.12);
    color: #8b5718;
  }
</style>
