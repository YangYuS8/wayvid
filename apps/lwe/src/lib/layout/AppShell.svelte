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

  const navLinkClass = (current: boolean) =>
    ['lwe-nav-link', current ? 'lwe-nav-link-active' : 'lwe-nav-link-idle'].join(' ');
</script>

<a class="lwe-skip-link" href="#app-content">Skip to content</a>

<div class="lwe-shell-bg">
  <div class="lwe-shell-grid">
    <aside class="lwe-shell-sidebar" aria-label="Primary">
      <div class="grid gap-2 pb-1">
        <p class="lwe-kicker">LWE</p>
        <p class="m-0 text-[1.35rem] font-semibold tracking-tight text-white">Wallpaper Engine</p>
        <p class="m-0 text-sm leading-6 text-slate-300">
          A persistent shell for library, workshop, desktop, and settings workflows.
        </p>
      </div>

      <nav class="grid gap-3" aria-label="Primary navigation">
        {#each navItems as item}
          {@const current = isCurrent(item.href, currentPath)}

          <a class={navLinkClass(current)} href={item.href} aria-current={current ? 'page' : undefined}>
            <span class="text-base font-semibold text-white">{item.label}</span>
            <span class="text-[0.68rem] font-semibold uppercase tracking-[0.2em] text-slate-300">
              {item.shortLabel}
            </span>
            <span class="text-sm leading-6 text-slate-300/90">{item.description}</span>
          </a>
        {/each}
      </nav>
    </aside>

    <main class="lwe-shell-main" id="app-content" tabindex="-1">
      {@render children?.()}
    </main>
  </div>
</div>
