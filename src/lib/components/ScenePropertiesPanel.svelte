<script lang="ts">
  import { onDestroy, untrack } from 'svelte';
  import { Button } from '$lib/ui/button';
  import { copy } from '$lib/i18n';
  import { loadSceneProperties, updateSceneProperties } from '$lib/ipc';
  import type { ActionOutcome, ScenePropertiesSnapshot, ScenePropertyValue } from '$lib/types';

  interface Props {
    itemId: string;
    monitorId: string;
    disabled?: boolean;
    onOutcome?: (outcome: ActionOutcome<null>) => void;
  }

  let { itemId, monitorId, disabled = false, onOutcome }: Props = $props();
  const panelId = $props.id();
  let expanded = $state(false);
  let snapshot = $state<ScenePropertiesSnapshot | null>(null);
  let draft = $state<Record<string, boolean | string | [number, number, number]>>({});
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let refreshError = $state<string | null>(null);
  let message = $state<string | null>(null);
  let activeKey = '';
  let requestToken = 0;
  let destroyed = false;

  const selectionKey = $derived(JSON.stringify([itemId, monitorId]));
  const panelCopy = $derived($copy.components.sceneProperties);
  const controlsDisabled = $derived(disabled || loading || saving);
  const values = $derived(readDraft());

  $effect(() => {
    const key = selectionKey;
    untrack(() => {
      if (activeKey === key) return;
      activeKey = key;
      requestToken += 1;
      expanded = false;
      snapshot = null;
      draft = {};
      loading = false;
      saving = false;
      error = null;
      refreshError = null;
      message = null;
    });
  });

  onDestroy(() => { destroyed = true; requestToken += 1; });

  const isCurrent = (token: number, key: string) =>
    !destroyed && requestToken === token && selectionKey === key;
  const finiteOrNull = (value: unknown) => value === null || (typeof value === 'number' && Number.isFinite(value));
  const isColor = (value: unknown): value is [number, number, number] =>
    Array.isArray(value) && value.length === 3 && value.every((channel) =>
      typeof channel === 'number' && Number.isFinite(channel) && channel >= 0 && channel <= 1);

  function validSnapshot(value: unknown, expectedItem: string, expectedMonitor: string): value is ScenePropertiesSnapshot {
    if (!value || typeof value !== 'object') return false;
    const candidate = value as ScenePropertiesSnapshot;
    if (candidate.itemId !== expectedItem || candidate.monitorId !== expectedMonitor || !Array.isArray(candidate.properties)) return false;
    const names = new Set<string>();
    return candidate.properties.every((property) => {
      if (!property || typeof property !== 'object' || typeof property.name !== 'string' || !property.name ||
          typeof property.label !== 'string' || names.has(property.name)) return false;
      names.add(property.name);
      if (![property.min, property.max, property.step, property.order].every(finiteOrNull) ||
          (property.min !== null && property.max !== null && property.min > property.max) ||
          (property.step !== null && property.step <= 0)) return false;
      if (property.kind === 'bool') return typeof property.value === 'boolean';
      if (property.kind === 'color') return isColor(property.value);
      if (property.kind === 'combo') {
        if (typeof property.value !== 'string' || !Array.isArray(property.options) ||
            property.options.length === 0 || property.options.length > 256) return false;
        const options = new Set<string>();
        return property.options.every((option) => {
          if (!option || typeof option !== 'object' || typeof option.label !== 'string' ||
              typeof option.value !== 'string' || !option.value || options.has(option.value)) return false;
          options.add(option.value);
          return true;
        }) && options.has(property.value);
      }
      return property.kind === 'slider' && typeof property.value === 'number' && Number.isFinite(property.value) &&
        (property.min === null || property.value >= property.min) &&
        (property.max === null || property.value <= property.max);
    });
  }

  function readDraft(): Record<string, ScenePropertyValue> | null {
    if (!snapshot) return null;
    const entries: [string, ScenePropertyValue][] = [];
    for (const property of snapshot.properties) {
      const value = draft[property.name];
      if (property.kind === 'bool') {
        if (typeof value !== 'boolean') return null;
        entries.push([property.name, value]);
      } else if (property.kind === 'color') {
        if (!isColor(value)) return null;
        entries.push([property.name, value]);
      } else if (property.kind === 'combo') {
        if (typeof value !== 'string' || !property.options.some((option) => option.value === value)) return null;
        entries.push([property.name, value]);
      } else {
        if (typeof value !== 'string' || !value.trim()) return null;
        const number = Number(value);
        if (!Number.isFinite(number) || (property.min !== null && number < property.min) ||
            (property.max !== null && number > property.max)) return null;
        entries.push([property.name, number]);
      }
    }
    return Object.fromEntries(entries);
  }

  function edit(name: string, value: boolean | string | [number, number, number]) {
    draft = { ...draft, [name]: value };
    error = null;
    message = null;
  }

  const hexColor = (color: [number, number, number]) => '#' + color.map((channel) =>
    Math.round(channel * 255).toString(16).padStart(2, '0')).join('');
  const rgbColor = (hex: string): [number, number, number] => [
    parseInt(hex.slice(1, 3), 16) / 255,
    parseInt(hex.slice(3, 5), 16) / 255,
    parseInt(hex.slice(5, 7), 16) / 255
  ];
  const errorText = (reason: unknown) => reason instanceof Error ? reason.message : String(reason);

  async function fetchSnapshot(token: number, key: string, currentItem: string, currentMonitor: string) {
    const result = await loadSceneProperties(currentMonitor, currentItem);
    if (!isCurrent(token, key)) return;
    if (!validSnapshot(result, currentItem, currentMonitor)) throw new Error(panelCopy.invalidSnapshot);
    snapshot = result;
    draft = Object.fromEntries(result.properties.map((property) => [property.name,
      property.kind === 'slider' ? String(property.value) : property.kind === 'color' ? [...property.value] : property.value
    ]));
  }

  async function reload() {
    if (loading || saving || disabled) return;
    const token = ++requestToken;
    const key = selectionKey;
    const currentItem = itemId;
    const currentMonitor = monitorId;
    loading = true;
    error = null;
    refreshError = null;
    try { await fetchSnapshot(token, key, currentItem, currentMonitor); }
    catch (reason) {
      if (isCurrent(token, key)) {
        if (snapshot) refreshError = errorText(reason);
        else error = errorText(reason);
      }
    } finally { if (isCurrent(token, key)) loading = false; }
  }

  function toggle() {
    if (disabled || saving) return;
    expanded = !expanded;
    if (expanded && !snapshot && !loading) void reload();
  }

  async function save(reset: boolean) {
    if (controlsDisabled || (!reset && (!snapshot || !values))) return;
    const submitted = reset ? {} : values!;
    const token = ++requestToken;
    const key = selectionKey;
    const currentItem = itemId;
    const currentMonitor = monitorId;
    saving = true;
    error = null;
    refreshError = null;
    message = null;
    try {
      const outcome = await updateSceneProperties(currentMonitor, currentItem, submitted);
      if (!outcome || typeof outcome.ok !== 'boolean') throw new Error(panelCopy.invalidOutcome);
      // The mutation can complete after a selection change; its invalidations still matter.
      if (!destroyed) onOutcome?.(outcome);
      if (!isCurrent(token, key)) return;
      if (outcome.ok !== true) { error = outcome.message ?? $copy.actionFeedback.failed; return; }
      message = outcome.message ?? panelCopy.saved;
      loading = true;
      try { await fetchSnapshot(token, key, currentItem, currentMonitor); }
      catch (reason) { if (isCurrent(token, key)) refreshError = errorText(reason); }
    } catch (reason) { if (isCurrent(token, key)) error = errorText(reason); }
    finally { if (isCurrent(token, key)) { saving = false; loading = false; } }
  }
</script>

<section class="lwe-subpanel gap-3" data-detail-section="scene-properties" aria-busy={loading || saving}>
  <Button variant="outline" class="h-auto min-h-9 justify-between whitespace-normal text-left" onclick={toggle}
    aria-label={panelCopy.title} aria-expanded={expanded} aria-controls={panelId} disabled={disabled || saving}>
    {panelCopy.title}<span aria-hidden="true">{expanded ? '−' : '+'}</span>
  </Button>
  {#if expanded}
    <div id={panelId} class="grid min-w-0 gap-3">
      <p class="text-sm leading-6 text-muted-foreground">{panelCopy.description}</p>
      {#if loading}<p class="text-sm text-muted-foreground" role="status">{panelCopy.loading}</p>{/if}
      {#if snapshot}
        {#if snapshot.properties.length === 0}
          <p class="text-sm text-muted-foreground">{panelCopy.empty}</p>
        {:else}
          <fieldset class="grid min-w-0 gap-3" disabled={controlsDisabled}>
            <legend class="sr-only">{panelCopy.title}</legend>
            {#each snapshot.properties as property, index (property.name)}
              {@const inputId = `${panelId}-${index}`}
              {#if property.kind === 'bool'}
                <label class="flex min-w-0 items-start gap-2 text-sm" for={inputId}>
                  <input id={inputId} type="checkbox" checked={draft[property.name] as boolean} disabled={controlsDisabled}
                    onchange={(event) => edit(property.name, event.currentTarget.checked)}
                    class="mt-1 h-4 w-4 shrink-0 rounded border-input focus:ring-ring" />
                  <span class="lwe-wrap-safe">{property.label}</span>
                </label>
              {:else if property.kind === 'slider'}
                <label class="grid min-w-0 gap-1.5 text-sm" for={inputId}>
                  <span class="lwe-wrap-safe">{property.label}</span>
                  <input id={inputId} type="number" value={draft[property.name] as string} disabled={controlsDisabled}
                    min={property.min ?? undefined} max={property.max ?? undefined} step={property.step ?? 'any'}
                    oninput={(event) => edit(property.name, event.currentTarget.value)}
                    class="h-9 w-full min-w-0 rounded-md border border-input bg-background px-3 outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50" />
                </label>
              {:else if property.kind === 'combo'}
                <label class="grid min-w-0 gap-1.5 text-sm" for={inputId}>
                  <span class="lwe-wrap-safe">{property.label}</span>
                  <select id={inputId} value={draft[property.name] as string} disabled={controlsDisabled}
                    onchange={(event) => edit(property.name, event.currentTarget.value)}
                    class="h-9 w-full min-w-0 rounded-md border border-input bg-background px-3 outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50">
                    {#each property.options as option (option.value)}
                      <option value={option.value}>{option.label}</option>
                    {/each}
                  </select>
                </label>
              {:else}
                <label class="grid min-w-0 gap-1.5 text-sm" for={inputId}>
                  <span class="lwe-wrap-safe">{property.label}</span>
                  <input id={inputId} type="color" value={hexColor(draft[property.name] as [number, number, number])}
                    disabled={controlsDisabled} oninput={(event) => edit(property.name, rgbColor(event.currentTarget.value))}
                    class="h-9 w-20 rounded-md border border-input bg-background p-1 focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50" />
                </label>
              {/if}
            {/each}
          </fieldset>
          {#if !values}<p class="lwe-warning-banner lwe-wrap-safe" role="alert">{panelCopy.invalidValues}</p>{/if}
          <div class="flex flex-wrap gap-2">
            <Button onclick={() => save(false)} disabled={controlsDisabled || !values}>{saving ? panelCopy.saving : panelCopy.save}</Button>
            <Button variant="outline" onclick={() => save(true)} disabled={controlsDisabled}>{panelCopy.reset}</Button>
          </div>
        {/if}
      {/if}
      {#if error}
        <p class="lwe-warning-banner lwe-wrap-safe" role="alert">{error}</p>
        {#if !snapshot}
          <div class="flex flex-wrap gap-2">
            <Button variant="outline" onclick={reload} disabled={controlsDisabled}>{$copy.actionFeedback.retry}</Button>
            <Button variant="outline" onclick={() => save(true)} disabled={controlsDisabled}>{panelCopy.reset}</Button>
          </div>
        {/if}
      {/if}
      {#if message}<p class="lwe-info-banner lwe-wrap-safe" role="status">{message}</p>{/if}
      {#if refreshError}
        <p class="lwe-warning-banner lwe-wrap-safe" role="alert">{panelCopy.refreshFailed} {refreshError}</p>
        <Button variant="outline" onclick={reload} disabled={controlsDisabled}>{$copy.actionFeedback.retryRefresh}</Button>
      {/if}
    </div>
  {/if}
</section>
