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
    class="grid h-8 w-8 cursor-pointer place-items-center rounded-full border border-border/80 bg-card/80 text-xs font-semibold tracking-[0.12em] text-muted-foreground shadow-sm backdrop-blur transition hover:border-border hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
    aria-label={triggerAriaLabel}
    aria-expanded={open}
    onclick={toggleDisclosure}
  >
    ...
  </button>

  {#if open}
    <div class="absolute right-0 top-10 z-30 grid min-w-[14rem] gap-2 rounded-xl border border-border/80 bg-popover p-3 text-popover-foreground shadow-[0_20px_40px_rgba(15,23,42,0.16)]">
      <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{menuCopy.title}</p>

      <button
        type="button"
        class="rounded-lg px-3 py-2 text-left text-sm font-medium text-foreground transition hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        onclick={runApplyShortcut}
      >
        {menuCopy.applyFromDetails}
      </button>

      <p class="px-3 pb-1 text-xs leading-5 text-muted-foreground">
        {menuCopy.applyFromDetailsDescription}
      </p>
    </div>
  {/if}
</div>
