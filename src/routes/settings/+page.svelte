<script lang="ts">
  import { onMount } from 'svelte';
  import PageHeader from '$lib/layout/PageHeader.svelte';
  import { copy, setPreferredLanguage } from '$lib/i18n';
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
    error instanceof Error ? error.message : $copy.settings.requestError;

  type SettingsDraft = {
    language: string;
    theme: string;
    launchOnLogin: boolean;
  };

  const languageOptions = [
    { value: 'en', labelKey: 'en' },
    { value: 'zh-CN', labelKey: 'zh-CN' },
    { value: 'system', labelKey: 'system' }
  ] as const;

  const themeOptions = [
    { value: 'system', labelKey: 'system' },
    { value: 'light', labelKey: 'light' },
    { value: 'dark', labelKey: 'dark' }
  ] as const;

  const createDraft = (snapshot: SettingsPageSnapshot): SettingsDraft => ({
    language: snapshot.language,
    theme: snapshot.theme,
    launchOnLogin: snapshot.launchOnLogin
  });

  const applySnapshot = (snapshot: SettingsPageSnapshot) => {
    setSettingsSnapshot(snapshot);
    setPreferredLanguage(snapshot.language as 'en' | 'zh-CN' | 'system');
    draftSource = snapshot;
    draft = createDraft(snapshot);
  };

  const languageLabel = (value: string) => $copy.settings.languageOptions[value as 'en' | 'zh-CN' | 'system'] ?? value;
  const themeLabel = (value: string) => $copy.settings.themeOptions[value as 'system' | 'light' | 'dark'] ?? value;

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
  <title>{$copy.settings.pageTitle}</title>
</svelte:head>

<section class="grid gap-6">
  <PageHeader
    eyebrow={$copy.settings.pageTitle}
    title={$copy.settings.headerTitle}
    subtitle={$copy.settings.headerSubtitle}
  />

  {#if pageError}
    <p class="lwe-warning-banner" role="alert" aria-live="assertive">{pageError}</p>
  {:else if loading && !hasSnapshot}
    <p class="text-sm text-slate-600" role="status" aria-live="polite">{$copy.settings.loading}</p>
  {:else if snapshot}
    <div class="grid gap-5 xl:grid-cols-[minmax(0,1.3fr)_minmax(280px,0.9fr)] xl:items-start">
      <Card class="lwe-panel gap-5">
        <div class="grid gap-1.5">
          <p class="lwe-eyebrow">{isEditing ? $copy.settings.editableSettings : $copy.settings.currentSettings}</p>
          <h2 class="lwe-heading-md">{$copy.settings.preferences}</h2>
          <p class="text-sm leading-6 text-slate-600">
            {#if isEditing}
              {$copy.settings.editSubtitle}
            {:else}
              {$copy.settings.reviewSubtitle}
            {/if}
          </p>
        </div>

        {#if actionMessage}
          <p class="lwe-info-banner" role="status" aria-live="polite">{actionMessage}</p>
        {/if}

        {#if isEditing}
          <div class="grid gap-4">
            <label class="grid gap-1.5">
              <span class="lwe-eyebrow">{$copy.settings.language}</span>
              <Select.Root type="single" name="settingsLanguage" bind:value={draft.language}>
                <Select.Trigger aria-label={$copy.settings.language} class="min-w-[14rem]">
                  {languageLabel(draft.language)}
                </Select.Trigger>

                <Select.Content>
                  {#each languageOptions as option}
                    <Select.Item value={option.value} label={languageLabel(option.value)}>{languageLabel(option.value)}</Select.Item>
                  {/each}
                </Select.Content>
              </Select.Root>
            </label>

            <label class="grid gap-1.5">
              <span class="lwe-eyebrow">{$copy.settings.theme}</span>
              <Select.Root type="single" name="settingsTheme" bind:value={draft.theme}>
                <Select.Trigger aria-label={$copy.settings.theme} class="min-w-[14rem]">
                  {themeLabel(draft.theme)}
                </Select.Trigger>

                <Select.Content>
                  {#each themeOptions as option}
                    <Select.Item value={option.value} label={themeLabel(option.value)}>{themeLabel(option.value)}</Select.Item>
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
                  aria-label={$copy.settings.launchOnLogin}
                  class="mt-1 h-4 w-4 rounded border-slate-300 text-sky-600 focus:ring-sky-400"
                />
                <span class="grid gap-1.5">
                  <span class="lwe-eyebrow">{$copy.settings.launchOnLogin}</span>
                  <span class="text-sm leading-6 text-slate-700">
                    {$copy.settings.startOnSession}
                  </span>
                </span>
              </label>
            {:else}
              <div class="grid gap-2 rounded-[1rem] border border-dashed border-slate-200/80 bg-slate-50/60 p-4">
                <p class="lwe-eyebrow">{$copy.settings.launchOnLogin}</p>
                <p class="text-sm leading-6 text-slate-700">
                  {$copy.settings.launchOnLoginUnavailable}
                </p>
                <p class="text-sm leading-6 text-slate-600">
                  {$copy.settings.launchPreferencePrefix} {draft.launchOnLogin ? $copy.settings.preferEnabled.toLowerCase() : $copy.settings.preferDisabled.toLowerCase()}.
                </p>
              </div>
            {/if}
          </div>

          <div class="flex flex-wrap items-center gap-3">
            <Button onclick={saveSettings} disabled={!hasChanges || saving}>
              {saving ? $copy.settings.saving : $copy.settings.saveChanges}
            </Button>
            <Button onclick={cancelEditing} variant="outline" disabled={saving}>{$copy.settings.cancel}</Button>

            {#if hasChanges && !saving}
              <p class="text-sm leading-6 text-slate-600" aria-live="polite">{$copy.settings.unsavedChanges}</p>
            {/if}
          </div>
        {:else}
          <div class="grid gap-4 rounded-[1rem] border border-slate-200/80 bg-slate-50/60 p-4 text-sm leading-6 text-slate-700">
            <p><span class="font-medium text-slate-950">{$copy.settings.language}:</span> {languageLabel(snapshot.language)}</p>
            <p><span class="font-medium text-slate-950">{$copy.settings.theme}:</span> {themeLabel(snapshot.theme)}</p>
            <p>
              <span class="font-medium text-slate-950">
                {snapshot.launchOnLoginAvailable ? $copy.settings.launchOnLoginSaved : $copy.settings.savedLaunchPreference}
              </span>
              {#if snapshot.launchOnLoginAvailable}
                {snapshot.launchOnLogin ? $copy.settings.enabled : $copy.settings.disabled}
              {:else}
                {snapshot.launchOnLogin ? $copy.settings.preferEnabled : $copy.settings.preferDisabled}
              {/if}
            </p>
          </div>

          <div class="flex flex-wrap items-center gap-3">
            <Button onclick={startEditing}>{$copy.settings.editSettings}</Button>
          </div>
        {/if}
      </Card>

      <Card class="lwe-panel gap-4">
        <div class="grid gap-1.5">
          <p class="lwe-eyebrow">{$copy.settings.steamIntegration}</p>
          <h2 class="lwe-heading-md">{$copy.settings.currentState}</h2>
        </div>

        <div class="lwe-subpanel gap-2.5">
          <p class="text-sm font-semibold tracking-tight text-slate-950">
            {snapshot.steamRequired ? $copy.settings.steamRequired : $copy.settings.steamOptional}
          </p>
          <p class="text-sm leading-6 text-slate-600">{snapshot.steamStatusMessage}</p>
        </div>

        <div class="grid gap-1.5 text-sm leading-6 text-slate-600">
          <p><span class="font-medium text-slate-950">{$copy.settings.savedLanguage}</span> {languageLabel(snapshot.language)}</p>
          <p><span class="font-medium text-slate-950">{$copy.settings.savedTheme}</span> {themeLabel(snapshot.theme)}</p>
          <p>
            <span class="font-medium text-slate-950">
              {snapshot.launchOnLoginAvailable ? $copy.settings.launchOnLoginSaved : $copy.settings.savedLaunchPreference}
            </span>
            {#if snapshot.launchOnLoginAvailable}
              {snapshot.launchOnLogin ? $copy.settings.enabled : $copy.settings.disabled}
            {:else}
              {snapshot.launchOnLogin ? $copy.settings.preferEnabled : $copy.settings.preferDisabled}
            {/if}
          </p>
        </div>
      </Card>
    </div>
  {/if}
</section>
