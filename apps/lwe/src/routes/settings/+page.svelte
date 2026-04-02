<script lang="ts">
  import { onMount } from 'svelte';
  import PageHeader from '$lib/layout/PageHeader.svelte';
  import { Button } from '$lib/ui/button';
  import { Card } from '$lib/ui/card';
  import * as Select from '$lib/ui/select';
  import { loadSettingsPage, updateSettings } from '$lib/ipc';
  import {
    needsPageLoad,
    pageCache,
    setCurrentPage,
    setSettingsSnapshot
  } from '$lib/stores/ui';
  import type { SettingsPageSnapshot } from '$lib/types';

  export let initialEditing = false;

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : 'Unable to complete the Settings request.';

  type SettingsDraft = {
    language: string;
    theme: string;
    launchOnLogin: boolean;
  };

  const languageOptions = [
    { value: 'en', label: 'English' },
    { value: 'zh-CN', label: 'Simplified Chinese' },
    { value: 'system', label: 'Follow system locale' }
  ] as const;

  const themeOptions = [
    { value: 'system', label: 'Follow system theme' },
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' }
  ] as const;

  const createDraft = (snapshot: SettingsPageSnapshot): SettingsDraft => ({
    language: snapshot.language,
    theme: snapshot.theme,
    launchOnLogin: snapshot.launchOnLogin
  });

  const applySnapshot = (snapshot: SettingsPageSnapshot) => {
    setSettingsSnapshot(snapshot);
    draftSource = snapshot;
    draft = createDraft(snapshot);
  };

  const labelFor = (options: readonly { value: string; label: string }[], value: string) =>
    options.find((option) => option.value === value)?.label ?? value;

  let loading = false;
  let saving = false;
  let pageError: string | null = null;
  let actionMessage: string | null = null;
  let draftSource: SettingsPageSnapshot | null = null;
  let isEditing = initialEditing;
  let draft: SettingsDraft = {
    language: 'en',
    theme: 'system',
    launchOnLogin: false
  };

  $: snapshot = $pageCache.settings.snapshot;
  $: if (snapshot && snapshot !== draftSource) {
    draft = createDraft(snapshot);
    draftSource = snapshot;
  }
  $: hasSnapshot = Boolean(snapshot);
  $: hasChanges =
    snapshot !== null &&
    (draft.language !== snapshot.language ||
      draft.theme !== snapshot.theme ||
      draft.launchOnLogin !== snapshot.launchOnLogin);

  const ensurePage = async () => {
    if (!needsPageLoad('settings')) {
      return;
    }

    loading = true;
    pageError = null;
    actionMessage = null;

    try {
      applySnapshot(await loadSettingsPage());
    } catch (error) {
      pageError = readError(error);
    } finally {
      loading = false;
    }
  };

  const saveSettings = async () => {
    if (!snapshot || !hasChanges) {
      return;
    }

    saving = true;
    pageError = null;
    actionMessage = null;

    try {
      const outcome = await updateSettings({
        language: draft.language !== snapshot.language ? draft.language : null,
        theme: draft.theme !== snapshot.theme ? draft.theme : null,
        launchOnLogin:
          draft.launchOnLogin !== snapshot.launchOnLogin ? draft.launchOnLogin : null
      });

      if (outcome.currentUpdate) {
        applySnapshot(outcome.currentUpdate);
      }

      actionMessage = outcome.message;
      isEditing = false;
    } catch (error) {
      pageError = readError(error);
    } finally {
      saving = false;
    }
  };

  const startEditing = () => {
    actionMessage = null;
    isEditing = true;
  };

  const cancelEditing = () => {
    if (!snapshot) {
      return;
    }

    draft = createDraft(snapshot);
    draftSource = snapshot;
    actionMessage = null;
    pageError = null;
    isEditing = false;
  };

  onMount(() => {
    setCurrentPage('settings');
    void ensurePage();
  });
</script>

<svelte:head>
  <title>Settings</title>
</svelte:head>

<section class="grid gap-6">
  <PageHeader
    eyebrow="Settings"
    title="App preferences"
    subtitle="Choose how LWE looks and behaves when you start your desktop session."
  />

  {#if pageError}
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !hasSnapshot}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">Loading Settings…</p>
  {:else if snapshot}
    <div class="grid gap-5 xl:grid-cols-[minmax(0,1.3fr)_minmax(280px,0.9fr)] xl:items-start">
      <Card class="lwe-panel gap-5">
        <div class="grid gap-1.5">
          <p class="lwe-eyebrow">{isEditing ? 'Editable settings' : 'Current settings'}</p>
          <h2 class="lwe-heading-md">Preferences</h2>
          <p class="text-sm leading-6 text-slate-600">
            {#if isEditing}
              Update language, theme, and startup behavior, then save when you are ready.
            {:else}
              Review the current language, theme, and startup behavior before making changes.
            {/if}
          </p>
        </div>

        {#if actionMessage}
          <p class="lwe-info-banner" role="status" aria-live="polite">{actionMessage}</p>
        {/if}

        {#if isEditing}
          <div class="grid gap-4">
            <label class="grid gap-1.5">
              <span class="lwe-eyebrow">Language</span>
              <Select.Root type="single" name="settingsLanguage" bind:value={draft.language}>
                <Select.Trigger aria-label="Language" class="min-w-[14rem]">
                  {labelFor(languageOptions, draft.language)}
                </Select.Trigger>

                <Select.Content>
                  {#each languageOptions as option}
                    <Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
                  {/each}
                </Select.Content>
              </Select.Root>
            </label>

            <label class="grid gap-1.5">
              <span class="lwe-eyebrow">Theme</span>
              <Select.Root type="single" name="settingsTheme" bind:value={draft.theme}>
                <Select.Trigger aria-label="Theme" class="min-w-[14rem]">
                  {labelFor(themeOptions, draft.theme)}
                </Select.Trigger>

                <Select.Content>
                  {#each themeOptions as option}
                    <Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
                  {/each}
                </Select.Content>
              </Select.Root>
            </label>

            {#if snapshot.launchOnLoginAvailable}
              <label class="flex items-start gap-3 rounded-[1rem] border border-slate-200/80 bg-slate-50/80 p-4">
                <input
                  type="checkbox"
                  bind:checked={draft.launchOnLogin}
                  disabled={saving}
                  aria-label="Launch on login"
                  class="mt-1 h-4 w-4 rounded border-slate-300 text-sky-600 focus:ring-sky-400"
                />
                <span class="grid gap-1.5">
                  <span class="lwe-eyebrow">Launch on login</span>
                  <span class="text-sm leading-6 text-slate-700">
                    Start LWE automatically when the graphical desktop session begins.
                  </span>
                </span>
              </label>
            {:else}
              <div class="grid gap-2 rounded-[1rem] border border-dashed border-slate-200/80 bg-slate-50/60 p-4">
                <p class="lwe-eyebrow">Launch on login</p>
                <p class="text-sm leading-6 text-slate-700">
                  Launch-on-login is currently unavailable on this machine.
                </p>
                <p class="text-sm leading-6 text-slate-600">
                  Saved preference: {draft.launchOnLogin ? 'prefer enabled when available' : 'prefer disabled when available'}.
                </p>
              </div>
            {/if}
          </div>

          <div class="flex flex-wrap items-center gap-3">
            <Button onclick={saveSettings} disabled={!hasChanges || saving}>
              {saving ? 'Saving…' : 'Save changes'}
            </Button>
            <Button onclick={cancelEditing} variant="outline" disabled={saving}>Cancel</Button>

            {#if hasChanges && !saving}
              <p class="text-sm leading-6 text-slate-600" aria-live="polite">Unsaved changes</p>
            {/if}
          </div>
        {:else}
          <div class="grid gap-4 rounded-[1rem] border border-slate-200/80 bg-slate-50/60 p-4 text-sm leading-6 text-slate-700">
            <p><span class="font-medium text-slate-950">Language:</span> {labelFor(languageOptions, snapshot.language)}</p>
            <p><span class="font-medium text-slate-950">Theme:</span> {labelFor(themeOptions, snapshot.theme)}</p>
            <p>
              <span class="font-medium text-slate-950">
                {snapshot.launchOnLoginAvailable ? 'Launch on login:' : 'Saved launch preference:'}
              </span>
              {#if snapshot.launchOnLoginAvailable}
                {snapshot.launchOnLogin ? 'enabled' : 'disabled'}
              {:else}
                {snapshot.launchOnLogin ? 'Prefer enabled when available' : 'Prefer disabled when available'}
              {/if}
            </p>
          </div>

          <div class="flex flex-wrap items-center gap-3">
            <Button onclick={startEditing}>Edit settings</Button>
          </div>
        {/if}
      </Card>

      <Card class="lwe-panel gap-4">
        <div class="grid gap-1.5">
          <p class="lwe-eyebrow">Steam integration</p>
          <h2 class="lwe-heading-md">Current state</h2>
        </div>

        <div class="lwe-subpanel gap-2.5">
          <p class="text-sm font-semibold tracking-tight text-slate-950">
            {snapshot.steamRequired ? 'Steam is required' : 'Steam is optional'}
          </p>
          <p class="text-sm leading-6 text-slate-600">{snapshot.steamStatusMessage}</p>
        </div>

        <div class="grid gap-1.5 text-sm leading-6 text-slate-600">
          <p><span class="font-medium text-slate-950">Saved language:</span> {labelFor(languageOptions, snapshot.language)}</p>
          <p><span class="font-medium text-slate-950">Saved theme:</span> {labelFor(themeOptions, snapshot.theme)}</p>
          <p>
            <span class="font-medium text-slate-950">
              {snapshot.launchOnLoginAvailable ? 'Launch on login:' : 'Saved launch preference:'}
            </span>
            {#if snapshot.launchOnLoginAvailable}
              {snapshot.launchOnLogin ? 'enabled' : 'disabled'}
            {:else}
              {snapshot.launchOnLogin ? 'Prefer enabled when available' : 'Prefer disabled when available'}
            {/if}
          </p>
        </div>
      </Card>
    </div>
  {/if}
</section>
