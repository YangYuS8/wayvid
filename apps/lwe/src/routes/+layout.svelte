<script lang="ts">
  import { page } from '$app/state';

  import AppShell from '$lib/layout/AppShell.svelte';
</script>

<AppShell currentPath={page.url.pathname}>
  <slot />
</AppShell>
