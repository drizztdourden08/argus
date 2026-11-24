/* @layer renderer-app @kind test */
import { screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import type { FakeSource } from '../helpers/fake-source';
import { openAlerts } from '../helpers/open-alerts';
import { renderAt } from '../helpers/render-at';
import { feed } from './helpers/feed';
import { CARDS } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { STALLS } from './helpers/stalls';
import { stubApi } from './helpers/stub-api';
import { cards, open } from './helpers/dashboards-world';

installHarness();
beforeEach(() => {
  stubApi();
});

describe('S1 the stalls on the overview card', { tags: ['standard'] }, () => {
  it('shows one red stalled chip on its project card, each open stall and its age in the title', async () => {
    await open({ stalls: STALLS });
    const [chip] = await screen.findAllByRole('button', { name: /^stalled: rotp / });
    const title = chip?.getAttribute('aria-label') ?? chip?.textContent ?? '';
    expect(title).toContain('stalled: rotp idle 12 min, 3 open items without a blocker (I-4, I-7, I-9) (open 12m)');
    expect(title).toContain('stalled: rotp did not read D-11 for 10 min (it is in a turn)');
    expect(screen.queryByRole('button', { name: /stalled: argus/ })).toBeNull();
  });

  it('shows no chip on a card without a stall', async () => {
    await open({ stalls: {} });
    await screen.findByText(cards.rotp.name);
    expect(screen.queryByRole('button', { name: /^stalled:/ })).toBeNull();
  });
});

describe('S1 the stalls in the board header', { tags: ['standard'] }, () => {
  it('shows the stalls of that board as chips with their age', async () => {
    const view = renderAt('/p/rotp/');
    await feed(await portfolioOf(view), { cards: CARDS, stalls: STALLS });
    const { panel } = await openAlerts();
    await waitFor(() => expect(within(panel).getAllByText(/^stalled:/)).toHaveLength(2));
    expect(within(panel).queryByText(/stalled: argus/)).toBeNull();
  });
});

describe('I-85 the item reference in a board header stall', { tags: ['standard'] }, () => {
  it('is a record link without the board prefix', async () => {
    const view = renderAt('/p/argus/');
    await feed(await portfolioOf(view), { cards: CARDS, stalls: STALLS });
    const { panel } = await openAlerts();
    const link = await within(panel).findByRole('link', { name: 'K-12' });
    expect(link.getAttribute('href')).toBe('/p/argus?open=K-12');
    expect(panel.textContent).toContain('stalled: K-12 doing 60 min, no agent');
  });
});
