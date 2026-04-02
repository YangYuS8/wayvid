<script lang="ts">
  import type { Snippet } from 'svelte';
  import { copy } from '$lib/i18n';

  type NavItem = {
    href: string;
    key: keyof (typeof $copy.appShell.nav);
  };

  const navItems: NavItem[] = [
    {
      href: '/library',
      key: 'library'
    },
    {
      href: '/workshop',
      key: 'workshop'
    },
    {
      href: '/desktop',
      key: 'desktop'
    },
    {
      href: '/settings',
      key: 'settings'
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

<a class="lwe-skip-link" href="#app-content">{$copy.appShell.skipToContent}</a>

<div class="lwe-shell-bg">
  <div class="lwe-shell-grid">
    <aside class="lwe-shell-sidebar" aria-label="Primary">
      <div class="grid gap-2 pb-1">
        <p class="lwe-kicker">LWE</p>
        <p class="m-0 text-[1.35rem] font-semibold tracking-tight text-white">Wallpaper Engine</p>
        <p class="m-0 text-sm leading-6 text-slate-300">
          {$copy.appShell.appDescription}
        </p>
      </div>

      <nav class="grid gap-3" aria-label="Primary navigation">
        {#each navItems as item}
          {@const current = isCurrent(item.href, currentPath)}
          {@const labels = $copy.appShell.nav[item.key]}

          <a class={navLinkClass(current)} href={item.href} aria-current={current ? 'page' : undefined}>
            <span class="text-base font-semibold text-white">{labels.label}</span>
            <span class="text-[0.68rem] font-semibold uppercase tracking-[0.2em] text-slate-300">
              {labels.shortLabel}
            </span>
            <span class="text-sm leading-6 text-slate-300/90">{labels.description}</span>
          </a>
        {/each}
      </nav>
    </aside>

    <main class="lwe-shell-main" id="app-content" tabindex="-1">
      {@render children?.()}
    </main>
  </div>
</div>
