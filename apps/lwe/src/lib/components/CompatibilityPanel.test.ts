import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import CompatibilityPanel from './CompatibilityPanel.svelte';

describe('CompatibilityPanel', () => {
  it('renders the explanation headline and detail', () => {
    const { body } = render(CompatibilityPanel, {
      props: {
        compatibility: {
          badge: 'unsupported',
          reasonCode: 'unsupported_web_item',
          headline: 'Web item not in first release',
          detail: 'Web Workshop items are recognized, but not yet supported.',
          nextStep: 'wait_for_future_support'
        }
      }
    });

    expect(body).toContain('Web item not in first release');
    expect(body).toContain('Web Workshop items are recognized, but not yet supported.');
  });
});
