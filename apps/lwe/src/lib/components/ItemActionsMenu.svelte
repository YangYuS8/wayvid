<script lang="ts">
  import { Button } from '$lib/ui/button';

  export let itemTitle: string;
  export let onApplyShortcut: (() => void) | undefined = undefined;

  let open = false;

  const toggleMenu = (event: MouseEvent) => {
    event.stopPropagation();
    open = !open;
  };

  const runApplyShortcut = (event: MouseEvent) => {
    event.stopPropagation();
    open = false;
    onApplyShortcut?.();
  };
</script>

<div class="relative pointer-events-auto">
  <Button
    variant="secondary"
    size="sm"
    class="h-8 rounded-full border border-slate-200/80 bg-white/90 px-3 text-xs font-semibold text-slate-700 shadow-sm backdrop-blur"
    aria-label={`Open quick actions for ${itemTitle}`}
    aria-expanded={open}
    aria-haspopup="menu"
    onclick={toggleMenu}
  >
    More actions
  </Button>

  {#if open}
    <div
      class="absolute right-0 top-10 z-30 grid min-w-[14rem] gap-1 rounded-xl border border-slate-200/80 bg-white p-2 shadow-[0_20px_40px_rgba(15,23,42,0.16)]"
      role="menu"
      aria-label={`Quick actions for ${itemTitle}`}
    >
      <button
        type="button"
        class="rounded-lg px-3 py-2 text-left text-sm font-medium text-slate-900 transition hover:bg-slate-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
        role="menuitem"
        onclick={runApplyShortcut}
      >
        Apply from details
      </button>

      <p class="px-3 pb-1 text-xs leading-5 text-slate-500">
        Opens the selected item in the detail panel so you can pick a monitor and apply it.
      </p>
    </div>
  {/if}
</div>
