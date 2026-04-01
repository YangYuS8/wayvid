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
    [
      'grid gap-2.5 rounded-[1.15rem] border px-4 py-4 text-left transition duration-150',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-300/70 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950',
      current
        ? 'border-sky-300/50 bg-gradient-to-b from-sky-400/25 to-sky-400/10 shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]'
        : 'border-transparent bg-white/5 hover:-translate-y-0.5 hover:border-sky-300/25 hover:bg-white/10'
    ].join(' ');
</script>

<div
  class="min-h-screen bg-[radial-gradient(circle_at_top,rgba(103,160,255,0.18),transparent_34%),linear-gradient(180deg,#eef3fb_0%,#e5ebf4_100%)] text-slate-950"
>
  <div class="grid min-h-screen gap-4 p-4 lg:grid-cols-[minmax(248px,292px)_minmax(0,1fr)] lg:gap-6 lg:p-5">
    <aside
      class="grid content-start gap-5 rounded-[1.75rem] border border-white/10 bg-slate-950/90 p-5 text-slate-50 shadow-[0_24px_56px_rgba(15,23,42,0.24)]"
      aria-label="Primary"
    >
      <div class="grid gap-2 pb-1">
        <p class="m-0 text-[0.72rem] font-semibold uppercase tracking-[0.24em] text-sky-100/70">LWE</p>
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

    <main
      class="grid min-w-0 content-start rounded-[2rem] border border-white/60 bg-white/72 p-5 shadow-[inset_0_1px_0_rgba(255,255,255,0.5),0_24px_60px_rgba(15,23,42,0.08)] backdrop-blur-xl sm:p-6"
      id="app-content"
    >
      {@render children?.()}
    </main>
  </div>
</div>
