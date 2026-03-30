<script lang="ts">
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { CompatibilityExplanationModel } from '$lib/types';

  const nextStepLabel = (nextStep: CompatibilityExplanationModel['nextStep']) => {
    switch (nextStep) {
      case 'none':
        return null;
      case 'open_in_steam':
        return 'Open this item in Steam.';
      case 'resync_workshop_item':
        return 'Resync this Workshop item to restore the missing files.';
      case 'wait_for_future_support':
        return 'Support for this item is planned for a future update.';
      default:
        return 'Check the item source for the next required action.';
    }
  };

  export let compatibility: CompatibilityExplanationModel;

  $: nextStep = nextStepLabel(compatibility.nextStep);
</script>

<section class="compatibility-panel" aria-label="Compatibility explanation">
  <div class="header">
    <StatusBadge label={compatibility.badge} />
  </div>

  <div class="copy">
    <h3>{compatibility.headline}</h3>
    <p>{compatibility.detail}</p>
  </div>

  {#if nextStep}
    <p class="next-step">Next step: {nextStep}</p>
  {/if}
</section>

<style>
  .compatibility-panel,
  .header,
  .copy {
    display: grid;
    gap: 0.7rem;
  }

  .compatibility-panel {
    padding: 0.9rem;
    border-radius: 16px;
    background: rgba(35, 69, 110, 0.04);
    border: 1px solid rgba(33, 52, 72, 0.1);
  }

  .header {
    grid-template-columns: repeat(auto-fit, minmax(0, max-content));
    align-items: center;
  }

  h3,
  p {
    margin: 0;
    overflow-wrap: anywhere;
  }

  .next-step {
    color: #5a6978;
  }
</style>
