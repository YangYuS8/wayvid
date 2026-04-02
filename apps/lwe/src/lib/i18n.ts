import { derived, writable } from 'svelte/store';

export type PreferredLanguage = 'en' | 'zh-CN' | 'system';
export type SupportedLocale = 'en' | 'zh-CN';

const dictionaries = {
  en: {
    appShell: {
      skipToContent: 'Skip to content',
      appDescription: 'A persistent shell for library, workshop, desktop, and settings workflows.',
      nav: {
        library: { label: 'Library', shortLabel: 'Browse', description: 'Review local projection state and assignments.' },
        workshop: { label: 'Workshop', shortLabel: 'Sync', description: 'Track Steam-backed content and refresh actions.' },
        desktop: { label: 'Desktop', shortLabel: 'Output', description: 'Inspect current monitor state and restore details.' },
        settings: { label: 'Settings', shortLabel: 'Config', description: 'Review shell-level environment and runtime status.' }
      }
    },
    settings: {
      pageTitle: 'Settings',
      headerTitle: 'App preferences',
      headerSubtitle: 'Choose how LWE looks and behaves when you start your desktop session.',
      loading: 'Loading Settings…',
      requestError: 'Unable to complete the Settings request.',
      currentSettings: 'Current settings',
      editableSettings: 'Editable settings',
      preferences: 'Preferences',
      reviewSubtitle: 'Review the current language, theme, and startup behavior before making changes.',
      editSubtitle: 'Update language, theme, and startup behavior, then save when you are ready.',
      language: 'Language',
      theme: 'Theme',
      launchOnLogin: 'Launch on login',
      launchOnLoginUnavailable: 'Launch-on-login is currently unavailable on this machine.',
      launchPreferencePrefix: 'Saved preference:',
      startOnSession: 'Start LWE automatically when the graphical desktop session begins.',
      saveChanges: 'Save changes',
      saving: 'Saving…',
      cancel: 'Cancel',
      editSettings: 'Edit settings',
      unsavedChanges: 'Unsaved changes',
      steamIntegration: 'Steam integration',
      currentState: 'Current state',
      steamRequired: 'Steam is required',
      steamOptional: 'Steam is optional',
      savedLanguage: 'Saved language:',
      savedTheme: 'Saved theme:',
      launchOnLoginSaved: 'Launch on login:',
      savedLaunchPreference: 'Saved launch preference:',
      enabled: 'enabled',
      disabled: 'disabled',
      preferEnabled: 'Prefer enabled when available',
      preferDisabled: 'Prefer disabled when available',
      languageOptions: {
        en: 'English',
        'zh-CN': 'Simplified Chinese',
        system: 'Follow system locale'
      },
      themeOptions: {
        system: 'Follow system theme',
        light: 'Light',
        dark: 'Dark'
      }
    }
  },
  'zh-CN': {
    appShell: {
      skipToContent: '跳到内容',
      appDescription: '一个常驻的外壳应用，用于串联内容库、创意工坊、桌面和设置流程。',
      nav: {
        library: { label: '内容库', shortLabel: '浏览', description: '查看本地投影视图状态和当前分配。' },
        workshop: { label: '创意工坊', shortLabel: '同步', description: '跟踪基于 Steam 的内容与刷新操作。' },
        desktop: { label: '桌面', shortLabel: '输出', description: '检查当前显示器状态和恢复详情。' },
        settings: { label: '设置', shortLabel: '配置', description: '查看应用层环境和运行状态。' }
      }
    },
    settings: {
      pageTitle: '设置',
      headerTitle: '应用偏好',
      headerSubtitle: '选择 LWE 在桌面会话启动时的外观和行为。',
      loading: '正在加载设置…',
      requestError: '无法完成设置请求。',
      currentSettings: '当前设置',
      editableSettings: '编辑设置',
      preferences: '偏好',
      reviewSubtitle: '查看当前语言、主题和启动行为，再决定是否修改。',
      editSubtitle: '修改语言、主题和启动行为，准备好后再保存。',
      language: '语言',
      theme: '主题',
      launchOnLogin: '登录时启动',
      launchOnLoginUnavailable: '当前设备暂不支持登录时自动启动。',
      launchPreferencePrefix: '已保存偏好：',
      startOnSession: '在图形桌面会话开始时自动启动 LWE。',
      saveChanges: '保存更改',
      saving: '正在保存…',
      cancel: '取消',
      editSettings: '编辑设置',
      unsavedChanges: '有未保存的更改',
      steamIntegration: 'Steam 集成',
      currentState: '当前状态',
      steamRequired: '需要 Steam',
      steamOptional: 'Steam 可选',
      savedLanguage: '已保存语言：',
      savedTheme: '已保存主题：',
      launchOnLoginSaved: '登录时启动：',
      savedLaunchPreference: '已保存启动偏好：',
      enabled: '已启用',
      disabled: '已禁用',
      preferEnabled: '可用时优先启用',
      preferDisabled: '可用时优先禁用',
      languageOptions: {
        en: 'English',
        'zh-CN': '简体中文',
        system: '跟随系统语言'
      },
      themeOptions: {
        system: '跟随系统主题',
        light: '浅色',
        dark: '深色'
      }
    }
  }
} as const;

const preferredLanguage = writable<PreferredLanguage>('en');

export const locale = derived(preferredLanguage, ($preferredLanguage): SupportedLocale =>
  $preferredLanguage === 'zh-CN' ? 'zh-CN' : 'en'
);

export const copy = derived(locale, ($locale) => dictionaries[$locale]);

export const setPreferredLanguage = (language: PreferredLanguage) => {
  preferredLanguage.set(language);
};

export const resetPreferredLanguage = () => {
  preferredLanguage.set('en');
};
