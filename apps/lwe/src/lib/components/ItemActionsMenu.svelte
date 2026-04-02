<script lang="ts">
  import { copy } from '$lib/i18n';

  export let itemTitle: string;
  export let onApplyShortcut: (() => void) | undefined = undefined;

  let open = false;

  $: menuCopy = $copy.components.itemActionsMenu;
  $: triggerAriaLabel = menuCopy.triggerAriaLabel.replace('{itemTitle}', itemTitle);

  const toggleDisclosure = (event: MouseEvent) => {
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
  <button
    type="button"
    class="grid h-8 w-8 cursor-pointer place-items-center rounded-full border border-slate-200/70 bg-white/80 text-xs font-semibold tracking-[0.12em] text-slate-500 shadow-sm backdrop-blur transition hover:border-slate-300/80 hover:text-slate-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 focus-visible:ring-offset-2"
    aria-label={triggerAriaLabel}
    aria-expanded={open}
    onclick={toggleDisclosure}
  >
    ...
  </button>

  {#if open}
    <div class="absolute right-0 top-10 z-30 grid min-w-[14rem] gap-2 rounded-xl border border-slate-200/80 bg-white p-3 shadow-[0_20px_40px_rgba(15,23,42,0.16)]">
      <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-500">{menuCopy.title}</p>

      <button
        type="button"
        class="rounded-lg px-3 py-2 text-left text-sm font-medium text-slate-900 transition hover:bg-slate-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
        onclick={runApplyShortcut}
      >
        {menuCopy.applyFromDetails}
      </button>

      <p class="px-3 pb-1 text-xs leading-5 text-slate-500">
        {menuCopy.applyFromDetailsDescription}
      </p>
    </div>
  {/if}
</div>
