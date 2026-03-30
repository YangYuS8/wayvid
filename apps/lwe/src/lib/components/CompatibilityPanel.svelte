<script lang="ts">
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import type { CompatibilityExplanationModel } from '$lib/types';

  export let compatibility: CompatibilityExplanationModel;
</script>

<section class="compatibility-panel" aria-label="Compatibility explanation">
  <div class="header">
    <StatusBadge label={compatibility.badge} />
  </div>

  <div class="copy">
    <h3>{compatibility.headline}</h3>
    <p>{compatibility.detail}</p>
  </div>

  {#if compatibility.nextStepCopy}
    <p class="next-step">Next step: {compatibility.nextStepCopy}</p>
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
