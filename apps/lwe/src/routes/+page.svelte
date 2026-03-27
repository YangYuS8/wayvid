<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  onMount(() => {
    void goto('/library');
  });
</script>

<svelte:head>
  <title>LWE</title>
</svelte:head>

<h1>LWE</h1>
<p>Opening the Library shell.</p>
<p><a href="/library">Continue to Library</a></p>
