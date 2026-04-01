<script lang="ts">
  import { shouldRenderCoverImage } from '$lib/components/cover-image';

  export let coverPath: string | null = null;
  export let label = 'cover';

  let loadFailed = false;
  let previousCoverPath: string | null = null;

  $: if (coverPath !== previousCoverPath) {
    previousCoverPath = coverPath;
    loadFailed = false;
  }

  $: showCoverImage = shouldRenderCoverImage(coverPath, loadFailed);
</script>

{#if showCoverImage}
  <img
    class="block aspect-[16/9] w-full rounded-[1.35rem] border border-slate-200/80 bg-slate-100 object-cover shadow-[inset_0_1px_0_rgba(255,255,255,0.45),0_16px_40px_rgba(15,23,42,0.08)]"
    src={coverPath ?? undefined}
    alt={label}
    width="1600"
    height="900"
    loading="lazy"
    on:error={() => (loadFailed = true)}
  />
{:else}
  <div
    class="grid aspect-[16/9] w-full place-items-center rounded-[1.35rem] border border-dashed border-slate-300 bg-[linear-gradient(135deg,rgba(241,245,249,1),rgba(248,250,252,0.94),rgba(226,232,240,1))] px-4 text-center shadow-[inset_0_1px_0_rgba(255,255,255,0.5)]"
    aria-label={`${label} placeholder`}
  >
    <div class="grid gap-1.5">
      <span class="text-[0.68rem] font-semibold uppercase tracking-[0.2em] text-slate-500">No Cover</span>
      <span class="text-sm leading-6 text-slate-600">Artwork will appear here when available.</span>
    </div>
  </div>
{/if}
