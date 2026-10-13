import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import CompatibilityPanel from './CompatibilityPanel.svelte';
import { resetPreferredLanguage, setPreferredLanguage } from '$lib/i18n';

describe('CompatibilityPanel', () => {
  afterEach(resetPreferredLanguage);
  it('renders the explanation headline and detail', () => {
    const { body } = render(CompatibilityPanel, {
      props: {
        compatibility: {
          badge: 'unsupported',
          reasonCode: 'unsupported_web_item',
          summaryCopy: 'Web support coming later',
          headline: 'Web item not in first release',
          detail: 'Web Workshop items are recognized, but not yet supported.',
          nextStep: 'wait_for_future_support',
          nextStepCopy: 'Support for this item is planned for a future update.'
        }
      }
    });

    expect(body).toContain('Web item not in first release');
    expect(body).toContain('Web Workshop items are recognized, but not yet supported.');
    expect(body).toContain('Next step: Support for this item is planned for a future update.');
    expect(body).not.toContain('unsupported_web_item');
    expect(body).not.toContain('wait_for_future_support');
    expect(body).toContain('data-slot="card"');
    expect(body).toContain('data-slot="separator"');
  });

  it('omits the next-step block when no assembled guidance copy is provided', () => {
    const { body } = render(CompatibilityPanel, {
      props: {
        compatibility: {
          badge: 'fully_supported',
          reasonCode: 'ready_for_library',
          summaryCopy: 'Ready to use',
          headline: 'Ready to use',
          detail: 'This video item is synchronized locally and available for Library and desktop use.',
          nextStep: 'none',
          nextStepCopy: null
        }
      }
    });

    expect(body).not.toContain('Next step:');
  });

  it.each(['en', 'zh-CN'] as const)('shows explicit Scene blockers with accessible localized labels in %s', (language) => {
    setPreferredLanguage(language);
    const { body } = render(CompatibilityPanel, {
      props: {
        compatibility: {
          badge: 'partially_supported', reasonCode: 'scene_unsupported_features',
          summaryCopy: 'Scene features unavailable', headline: 'Scene features unavailable',
          detail: 'Unavailable feature', nextStep: 'wait_for_future_support',
          nextStepCopy: 'Future runtime support required',
          sceneAssessment: {
            runnable: false, summary: null,
            issues: [
              { code: 'unsupported_particles', detail: 'Unsupported Scene feature (particle): rain.json' },
              { code: 'unsupported_hierarchy', detail: 'Unsupported Scene feature (hierarchy): image parent bindings' },
              { code: 'unsupported_properties', detail: 'Unsupported Scene feature (property_binding): invalid user property binding' }
            ]
          }
        }
      }
    });
    expect(body).toContain(language === 'en' ? 'Scene features unavailable' : '场景特性不可用');
    expect(body).toContain(language === 'en' ? 'Particle systems' : '粒子系统');
    expect(body).toContain(language === 'en' ? 'Layer hierarchy or parent bindings' : '图层层级或父级绑定');
    expect(body).toContain(language === 'en' ? 'Unsupported or invalid user property bindings' : '不受支持或无效的用户属性绑定');
    expect(body).toContain(language === 'en' ? 'aria-label="Playback blockers"' : 'aria-label="播放阻断原因"');
    expect(body).toContain('rain.json');
    expect(body).toContain('lwe-wrap-safe');
    expect(body).not.toContain('unsupported_particles');
    expect(body).not.toContain('unsupported_hierarchy');
    expect(body).not.toContain('unsupported_properties');
  });
});
