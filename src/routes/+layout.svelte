<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import '../app.css';

  import { loadSettingsPage } from '$lib/ipc';
  import { setPreferredLanguage } from '$lib/i18n';
  import AppShell from '$lib/layout/AppShell.svelte';
  import { applyThemePreference, setSettingsSnapshot } from '$lib/stores/ui';

  onMount(() => {
    void loadSettingsPage()
      .then((snapshot) => {
        setSettingsSnapshot(snapshot);
        setPreferredLanguage(snapshot.language as 'en' | 'zh-CN' | 'system');
        applyThemePreference(snapshot.theme as 'light' | 'dark' | 'system');
      })
      .catch(() => {
        setPreferredLanguage('en');
        applyThemePreference('system');
      });
  });
</script>

<AppShell currentPath={page.url.pathname}>
  <slot />
</AppShell>
