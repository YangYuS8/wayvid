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

  const readError = (error: unknown) =>
    error instanceof Error ? error.message : 'Unable to complete the Settings request.';

  type SettingsDraft = {
    language: string;
    theme: string;
    launchOnLogin: boolean;
  };

  const languageOptions = [
    { value: 'en', label: 'English' },
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
    draft = createDraft(snapshot);
  };

  const labelFor = (options: readonly { value: string; label: string }[], value: string) =>
    options.find((option) => option.value === value)?.label ?? value;

  let loading = false;
  let saving = false;
  let pageError: string | null = null;
  let actionMessage: string | null = null;
  let draft: SettingsDraft = {
    language: 'en',
    theme: 'system',
    launchOnLogin: false
  };

  $: snapshot = $pageCache.settings.snapshot;
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
    } catch (error) {
      pageError = readError(error);
    } finally {
      saving = false;
    }
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
    subtitle="Adjust the MVP desktop preferences the Rust backend owns today, then save them explicitly."
  />

  {#if pageError}
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !hasSnapshot}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">Loading Settings…</p>
  {:else if snapshot}
    <div class="grid gap-5 xl:grid-cols-[minmax(0,1.3fr)_minmax(280px,0.9fr)] xl:items-start">
      <Card class="lwe-panel gap-5">
        <div class="grid gap-1.5">
          <p class="lwe-eyebrow">Editable settings</p>
          <h2 class="lwe-heading-md">Preferences</h2>
          <p class="text-sm leading-6 text-slate-600">
            Save language, theme, and launch behavior through the backend-owned settings file.
          </p>
        </div>

        {#if actionMessage}
          <p class="lwe-info-banner" role="status" aria-live="polite">{actionMessage}</p>
        {/if}

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

          <label class="flex items-start gap-3 rounded-[1rem] border border-slate-200/80 bg-slate-50/80 p-4">
            <input
              type="checkbox"
              bind:checked={draft.launchOnLogin}
              disabled={!snapshot.launchOnLoginAvailable || saving}
              aria-label="Launch on login"
              class="mt-1 h-4 w-4 rounded border-slate-300 text-sky-600 focus:ring-sky-400"
            />
            <span class="grid gap-1.5">
              <span class="lwe-eyebrow">Launch on login</span>
              <span class="text-sm leading-6 text-slate-700">
                {#if snapshot.launchOnLoginAvailable}
                  Start LWE automatically when the graphical desktop session begins.
                {:else}
                  Launch-on-login is currently unavailable on this machine.
                {/if}
              </span>
            </span>
          </label>
        </div>

        <div class="flex flex-wrap items-center gap-3">
          <Button onclick={saveSettings} disabled={!hasChanges || saving}>
            {saving ? 'Saving…' : 'Save changes'}
          </Button>

          {#if hasChanges && !saving}
            <p class="text-sm leading-6 text-slate-600" aria-live="polite">Unsaved changes</p>
          {/if}
        </div>
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
          <p><span class="font-medium text-slate-950">Saved language:</span> {snapshot.language}</p>
          <p><span class="font-medium text-slate-950">Saved theme:</span> {snapshot.theme}</p>
          <p>
            <span class="font-medium text-slate-950">Launch on login:</span>
            {snapshot.launchOnLogin ? 'enabled' : 'disabled'}
          </p>
          <p><span class="font-medium text-slate-950">Snapshot stale:</span> {snapshot.stale ? 'yes' : 'no'}</p>
        </div>
      </Card>
    </div>
  {/if}
</section>
