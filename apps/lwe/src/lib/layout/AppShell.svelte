<script lang="ts">
  import type { Snippet } from 'svelte';

  type NavItem = {
    href: string;
    label: string;
    shortLabel: string;
    description: string;
  };

  const navItems: NavItem[] = [
    {
      href: '/library',
      label: 'Library',
      shortLabel: 'Browse',
      description: 'Review local projection state and assignments.'
    },
    {
      href: '/workshop',
      label: 'Workshop',
      shortLabel: 'Sync',
      description: 'Track Steam-backed content and refresh actions.'
    },
    {
      href: '/desktop',
      label: 'Desktop',
      shortLabel: 'Output',
      description: 'Inspect current monitor state and restore details.'
    },
    {
      href: '/settings',
      label: 'Settings',
      shortLabel: 'Config',
      description: 'Review shell-level environment and runtime status.'
    }
  ];

  let {
    currentPath = '/library',
    children
  }: {
    currentPath?: string;
    children?: Snippet;
  } = $props();

  const isCurrent = (href: string, pathname: string) => pathname === href || pathname.startsWith(`${href}/`);
</script>

<div class="app-shell">
  <aside class="sidebar" aria-label="Primary">
    <div class="brand">
      <p class="kicker">LWE</p>
      <h1>Wallpaper Engine</h1>
      <p class="summary">A persistent shell for library, workshop, desktop, and settings workflows.</p>
    </div>

    <nav class="navigation" aria-label="Primary navigation">
      {#each navItems as item}
        {@const current = isCurrent(item.href, currentPath)}

        <a class:current href={item.href} aria-current={current ? 'page' : undefined}>
          <span class="nav-label">{item.label}</span>
          <span class="nav-meta">{item.shortLabel}</span>
          <span class="nav-description">{item.description}</span>
        </a>
      {/each}
    </nav>
  </aside>

  <main class="content" id="app-content">
    {@render children?.()}
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    min-height: 100vh;
    background:
      radial-gradient(circle at top, rgba(103, 160, 255, 0.18), transparent 34%),
      linear-gradient(180deg, #eef3fb 0%, #e5ebf4 100%);
    color: #132033;
    font-family: 'Segoe UI', sans-serif;
  }

  :global(a) {
    color: inherit;
  }

  .app-shell {
    min-height: 100vh;
    display: grid;
    grid-template-columns: minmax(248px, 292px) minmax(0, 1fr);
    gap: 1.5rem;
    padding: 1.25rem;
    box-sizing: border-box;
  }

  .sidebar,
  .brand,
  .navigation,
  .content,
  a {
    display: grid;
    gap: 0.9rem;
  }

  .sidebar {
    align-content: start;
    padding: 1.35rem;
    border-radius: 28px;
    background: rgba(14, 24, 40, 0.9);
    color: #f3f7fb;
    box-shadow: 0 20px 48px rgba(11, 18, 31, 0.18);
  }

  .brand {
    gap: 0.45rem;
    padding-bottom: 0.25rem;
  }

  .kicker,
  .summary,
  .nav-meta,
  .nav-description,
  h1 {
    margin: 0;
  }

  .kicker,
  .nav-meta {
    font-size: 0.74rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  h1 {
    font-size: 1.35rem;
  }

  .summary {
    color: rgba(243, 247, 251, 0.74);
    line-height: 1.45;
  }

  .navigation {
    gap: 0.7rem;
  }

  a {
    padding: 0.95rem 1rem;
    border-radius: 18px;
    text-decoration: none;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid transparent;
    transition:
      transform 120ms ease,
      background 120ms ease,
      border-color 120ms ease;
  }

  a:hover,
  a:focus-visible {
    transform: translateY(-1px);
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(157, 197, 255, 0.28);
  }

  a:focus-visible {
    outline: 2px solid rgba(157, 197, 255, 0.5);
    outline-offset: 2px;
  }

  a.current {
    background: linear-gradient(180deg, rgba(116, 170, 255, 0.28), rgba(116, 170, 255, 0.14));
    border-color: rgba(157, 197, 255, 0.45);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.05);
  }

  .nav-label {
    font-size: 1rem;
    font-weight: 600;
  }

  .nav-meta,
  .nav-description {
    color: rgba(243, 247, 251, 0.72);
  }

  .nav-description {
    line-height: 1.4;
  }

  .content {
    align-content: start;
    min-width: 0;
    padding: 1.5rem;
    border-radius: 32px;
    background: rgba(255, 255, 255, 0.72);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.45);
    backdrop-filter: blur(12px);
  }

  @media (max-width: 900px) {
    .app-shell {
      grid-template-columns: 1fr;
      gap: 1rem;
      padding: 1rem;
    }

    .sidebar,
    .content {
      border-radius: 24px;
    }
  }
</style>
