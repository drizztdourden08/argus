/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { openFilters } from '../helpers/filter-clauses';
import { ago, DAY, launcherDoc, MIN, runDoc, standaloneDoc } from './fixtures';
import { withDefs } from './list-dom';
import { mountBuilds } from './mount-builds';

cleanAfterEach();

afterEach(() => localStorage.clear());

const closedToggle = async (count: number): Promise<HTMLElement> => {
  await openFilters();
  return screen.findByRole('button', { name: `Show closed (${count})` });
};

describe('S1 the usage window of a standalone launcher', { tags: ['standard'] }, () => {
  const stale = () => withDefs([
    launcherDoc('master'),
    standaloneDoc('fresh-1'),
    standaloneDoc('old-2', { lastUsedAt: ago(9 * DAY) }),
    standaloneDoc('old-3', { lastUsedAt: ago(40 * DAY), useEvery: '2w', useEveryMs: 14 * DAY }),
  ]);

  it('counts the newest Play run as a use, even when the doc does not know it yet', async () => {
    const { change } = await mountBuilds(stale());
    await screen.findByText('stale 2');
    change('runs', 'R-1', runDoc('R-1', 'launcher-old-2', { queuedAt: ago(MIN) }));
    expect(await screen.findByText('stale 1')).toBeTruthy();
  });
});

describe('S1 retired launchers', { tags: ['standard'] }, () => {
  const retiredBoard = () => withDefs([
    launcherDoc('master'),
    launcherDoc('old-1', { retiredAt: ago(2 * DAY), retiredBy: 'worker', retireNote: 'merged into main' }),
  ]);

  it('remember the choice', async () => {
    await mountBuilds(retiredBoard());
    await screen.findByText('builds 1');
    fireEvent.click(await closedToggle(1));
    expect(localStorage.getItem('coord.lcretired')).toBe('1');
    fireEvent.click(await closedToggle(1));
    expect(localStorage.getItem('coord.lcretired')).toBe('0');
  });

  it('open at once when the choice was kept', async () => {
    localStorage.setItem('coord.lcretired', '1');
    await mountBuilds(retiredBoard());
    await screen.findByText('builds 1');
    expect((await closedToggle(1)).getAttribute('aria-pressed')).toBe('true');
  });
});
