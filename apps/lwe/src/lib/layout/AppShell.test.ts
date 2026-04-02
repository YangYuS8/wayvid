import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import AppShell from './AppShell.svelte';
import { resetPreferredLanguage, setPreferredLanguage } from '$lib/i18n';

describe('AppShell', () => {
  afterEach(() => {
    resetPreferredLanguage();
  });

  it('renders persistent navigation, current-page emphasis, and a main region', () => {
    const { body } = render(AppShell, {
      props: {
        currentPath: '/desktop'
      }
    });

    expect(body).toContain('Library');
    expect(body).toContain('Workshop');
    expect(body).toContain('Desktop');
    expect(body).toContain('Settings');
    expect(body).toContain('aria-current="page"');
    expect(body).toContain('<main');
    expect(body).not.toContain('<h1');
  });

  it('renders navigation in Simplified Chinese when zh-CN is active', () => {
    setPreferredLanguage('zh-CN');

    const { body } = render(AppShell, {
      props: {
        currentPath: '/desktop'
      }
    });

    expect(body).toContain('内容库');
    expect(body).toContain('创意工坊');
    expect(body).toContain('桌面');
    expect(body).toContain('设置');
  });

  it('uses utility-based shell classes instead of the older local style block', () => {
    const { body } = render(AppShell, {
      props: {
        currentPath: '/library'
      }
    });

    expect(body).toContain('Skip to content');
    expect(body).toContain('href="#app-content"');
    expect(body).toContain('tabindex="-1"');
    expect(body).toContain('lwe-shell-sidebar');
    expect(body).toContain('lwe-shell-grid');
    expect(body).not.toContain('<style>');
  });
});
