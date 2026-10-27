/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { ago, buildDoc, DAY, launcherDoc, MIN, runDoc, standaloneDoc } from './fixtures';
import { cardOf, withDefs } from './list-dom';
import { mountBuilds } from './mount-builds';

cleanAfterEach();

afterEach(() => localStorage.clear());

const waitForCard = async (container: HTMLElement, name: string): Promise<HTMLElement> => {
  await screen.findByText(/to start/);
  return cardOf(container, name);
};

const retiredToggle = (count: number): HTMLElement => screen.getByText(`Retired (${count})`).closest('summary') as HTMLElement;

describe('a launcher made for a bundle', () => {
  it('says for bundle <tag> with a link to the Tests tab and has no usage window', async () => {
    const { container } = await mountBuilds(withDefs(
      [launcherDoc('master'), launcherDoc('open-1', { bundle: 'open-1', joined: 'open-1' }), launcherDoc('shared-1', { joined: 'open-1' })],
      { builds: [buildDoc('open-1', { launcher: 'open-1' })] },
    ));
    const links = await screen.findAllByRole('link', { name: 'for bundle open-1' });
    expect(links).toHaveLength(2);
    expect(links[0]?.getAttribute('href')).toBe('#tests');
    expect(screen.getAllByText('ready')).toHaveLength(2);
    expect(container.textContent).not.toContain('expected every');
    expect(screen.queryByText('stale')).toBeNull();
  });
});

describe('the usage window of a standalone launcher', () => {
  const stale = () => withDefs([
    launcherDoc('master'),
    standaloneDoc('fresh-1'),
    standaloneDoc('old-2', { lastUsedAt: ago(9 * DAY) }),
    standaloneDoc('old-3', { lastUsedAt: ago(40 * DAY), useEvery: '2w', useEveryMs: 14 * DAY }),
  ]);

  it('shows the window; unused past it, a stale tag and the header count say so', async () => {
    const { container } = await mountBuilds(stale());
    expect(await screen.findByText('4 to start · 2 stale')).toBeTruthy();
    expect(within(cardOf(container, 'fresh-1')).getByText(/last used/)).toBeTruthy();
    expect(within(cardOf(container, 'fresh-1')).getByText(/expected every 7d/)).toBeTruthy();
    expect(within(cardOf(container, 'fresh-1')).queryByText('stale')).toBeNull();
    expect(within(cardOf(container, 'old-2')).getByText('stale')).toBeTruthy();
    expect(within(cardOf(container, 'old-2')).getByText(/unused/)).toBeTruthy();
    expect(within(cardOf(container, 'old-3')).getByText(/expected every 2w/)).toBeTruthy();
    expect(within(cardOf(container, 'master')).queryByText(/expected every/)).toBeNull();
  });

  it('counts the newest Play run as a use, even when the doc does not know it yet', async () => {
    const { change } = await mountBuilds(stale());
    await screen.findByText('4 to start · 2 stale');
    change('runs', 'R-1', runDoc('R-1', 'launcher-old-2', { queuedAt: ago(MIN) }));
    expect(await screen.findByText('4 to start · 1 stale')).toBeTruthy();
  });

  it('shows the 7d default of a launcher that has no useEvery', async () => {
    const { container } = await mountBuilds(withDefs([launcherDoc('master'), standaloneDoc('legacy-1', { useEvery: null, lastUsedAt: ago(10 * DAY) })]));
    const card = within(await waitForCard(container, 'legacy-1'));
    expect(card.getByText(/expected every 7d, default/)).toBeTruthy();
    expect(card.getByText(/expected every 7d, default/).getAttribute('title')).toContain('7d is assumed');
    expect(card.getByText('stale')).toBeTruthy();
  });

  it('a keep holds a launcher fresh until it ends', async () => {
    const { container } = await mountBuilds(withDefs([
      launcherDoc('master'),
      standaloneDoc('kept-1', { useEvery: '3d', useEveryMs: 3 * DAY, lastUsedAt: ago(10 * DAY), keptUntil: new Date(Date.now() + 2 * DAY).toISOString() }),
    ]));
    const card = within(await waitForCard(container, 'kept-1'));
    expect(card.queryByText('stale')).toBeNull();
    expect(screen.queryByText(/stale/)).toBeNull();
  });
});

describe('retired launchers', () => {
  const retiredBoard = () => withDefs([
    launcherDoc('master'),
    launcherDoc('old-1', { retiredAt: ago(2 * DAY), retiredBy: 'worker', retireNote: 'merged into main' }),
  ]);

  it('count in the header and sit behind a Retired toggle, folded', async () => {
    await mountBuilds(retiredBoard());
    expect(await screen.findByText('1 to start · 1 retired')).toBeTruthy();
    expect((retiredToggle(1).closest('details') as HTMLDetailsElement).open).toBe(false);
  });

  it('open to read-only rows: the retire note, who retired it, no Play, no usage window', async () => {
    const { container } = await mountBuilds(retiredBoard());
    await screen.findByText('1 to start · 1 retired');
    fireEvent.click(retiredToggle(1));
    expect((retiredToggle(1).closest('details') as HTMLDetailsElement).open).toBe(true);
    const card = within(cardOf(container, 'old-1'));
    expect(card.getByText('merged into main')).toBeTruthy();
    expect(card.getByText(/by worker/)).toBeTruthy();
    expect(card.queryByRole('button', { name: /Play/ })).toBeNull();
    expect(card.queryByText(/expected every/)).toBeNull();
  });

  it('remember the choice', async () => {
    await mountBuilds(retiredBoard());
    await screen.findByText('1 to start · 1 retired');
    fireEvent.click(retiredToggle(1));
    expect(localStorage.getItem('coord.lcretired')).toBe('1');
    fireEvent.click(retiredToggle(1));
    expect(localStorage.getItem('coord.lcretired')).toBe('0');
  });

  it('open at once when the choice was kept', async () => {
    localStorage.setItem('coord.lcretired', '1');
    await mountBuilds(retiredBoard());
    await screen.findByText('1 to start · 1 retired');
    expect((retiredToggle(1).closest('details') as HTMLDetailsElement).open).toBe(true);
  });

  it('have no toggle when none is retired', async () => {
    await mountBuilds(withDefs([launcherDoc('master')]));
    await screen.findByText('1 to start');
    expect(screen.queryByText(/^Retired \(/)).toBeNull();
  });
});
