/* @layer renderer-app @kind test */
import { screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { feed } from './helpers/feed';
import { CARDS, NOTIFICATIONS } from './helpers/fixtures';
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
    const [chip] = await screen.findAllByTitle(/^stalled: rotp idle 12 min/);
    const title = chip?.getAttribute('title') ?? '';
    expect(title).toContain('stalled: rotp idle 12 min, 3 open items without a blocker (I-4, I-7, I-9) (open 12m)');
    expect(title).toContain('stalled: rotp did not read D-11 for 10 min (it is in a turn)');
    expect(screen.queryByTitle(/stalled: argus/)).toBeNull();
  });

  it('shows no chip on a card without a stall', async () => {
    await open({ stalls: {} });
    await screen.findByText(cards.rotp.name);
    expect(screen.queryByTitle(/^stalled:/)).toBeNull();
  });
});

describe('S1 the stalls in the Notifications tab', { tags: ['standard'] }, () => {
  const openTab = async (stalls: Record<string, object>) => {
    const view = renderAt('/#notifications');
    await feed(view.sources, { cards: CARDS, notifications: NOTIFICATIONS, stalls });
    return view;
  };

  it('lists the stalls oldest first with the kind, the project and the age', async () => {
    await openTab(STALLS);
    const list = await screen.findByRole('list', { name: 'Stalls' });
    const rows = [...list.querySelectorAll<HTMLElement>('[data-record]')];
    expect(rows).toHaveLength(3);
    expect(rows[0]?.textContent).toContain('silent');
    expect(rows[0]?.textContent).toContain('1h');
    expect(rows[1]?.textContent).toContain('unread');
    expect(rows[2]?.textContent).toContain('idle');
    expect(within(rows[2] as HTMLElement).getByRole('link', { name: 'RotP' }).getAttribute('href')).toBe('/p/rotp/');
  });

  it('marks an acked stall with its note as the title', async () => {
    await openTab(STALLS);
    const list = await screen.findByRole('list', { name: 'Stalls' });
    const acked = within(list).getAllByText('acked');
    expect(acked).toHaveLength(1);
    expect(acked[0]?.closest('[title]')?.getAttribute('title')).toBe('asked the lead');
  });

  it('shows no Stalls list when none is open', async () => {
    await openTab({});
    await screen.findByText('RotP: build ready');
    expect(screen.queryByRole('list', { name: 'Stalls' })).toBeNull();
  });
});

describe('S1 the stalls in the board header', { tags: ['standard'] }, () => {
  it('shows the stalls of that board as chips with their age', async () => {
    const view = renderAt('/p/rotp/');
    await waitFor(() => expect(view.sources.length).toBeGreaterThan(0));
    await feed(view.sources, { cards: CARDS, stalls: STALLS });
    const header = await screen.findByRole('banner');
    await waitFor(() => expect(within(header).getAllByText(/^stalled:/)).toHaveLength(2));
    expect(within(header).queryByText(/stalled: argus/)).toBeNull();
  });
});

describe('I-85 the item reference in a board header stall', { tags: ['standard'] }, () => {
  it('is a record link without the board prefix', async () => {
    const view = renderAt('/p/argus/');
    await waitFor(() => expect(view.sources.length).toBeGreaterThan(0));
    await feed(view.sources, { cards: CARDS, stalls: STALLS });
    const header = await screen.findByRole('banner');
    const link = await within(header).findByRole('link', { name: 'K-12' });
    expect(link.getAttribute('href')).toBe('/p/argus?open=K-12');
    expect(header.textContent).toContain('stalled: K-12 doing 60 min, no agent');
  });
});
