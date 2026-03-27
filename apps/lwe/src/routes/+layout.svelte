<script lang="ts">
  const pages = [
    { href: '/library', label: 'Library' },
    { href: '/workshop', label: 'Workshop' },
    { href: '/desktop', label: 'Desktop' },
    { href: '/settings', label: 'Settings' }
  ];
</script>

<nav>
  {#each pages as page}
    <a href={page.href}>{page.label}</a>
  {/each}
</nav>

<slot />
