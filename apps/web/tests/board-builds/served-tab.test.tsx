/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { PANE_TABS } from '../../src/shell/records/pane/pane.constants';
import { servedBadge } from '../../src/shell/records/pane/model/served-badge';
import type { ItemRef } from '../../src/data/kanban';
import { itemRef } from '../board-kanban/kanban-docs';
import { paneData } from '../board-kanban/pane-data';
import { cleanAfterEach } from '../board-tests/board-dom';
import { worktree } from '../board-worktrees/fixtures';
import { ago, DAY, launcherDoc } from './fixtures';
import { withDefs } from './list-dom';
import { mountServed } from './mount-served';

cleanAfterEach();

const badge = (servedBy?: ItemRef['servedBy']) => servedBadge(paneData({ item: itemRef('I-1', servedBy ? { servedBy } : {}) }));

describe('P13 the Work tab of the item pane', { tags: ['critical'] }, () => {
  it('is registered as a pane tab with a badge', () => {
    expect(PANE_TABS.work?.label).toBe('Work');
    expect(PANE_TABS.work?.badge).toBeDefined();
  });

  it('counts live builds and worktrees, and hides at zero or without the count', () => {
    expect(badge({ builds: 2, closed: 1, worktrees: 1 })?.value).toBe(3);
    expect(badge({ builds: 0, closed: 2, worktrees: 0 })).toBeUndefined();
    expect(badge()).toBeUndefined();
  });

  it('lists the builds that serve the item, closed ones without Play, and the worktrees linked to it', async () => {
    const { container } = await mountServed('I-1', {
      ...withDefs([
        launcherDoc('web', { for: 'I-1' }),
        launcherDoc('done', { for: 'I-1', retiredAt: ago(DAY), retiredByItem: 'I-1', stage: 'closed' }),
        launcherDoc('other', { for: 'I-2' }),
        launcherDoc('master', { master: true }),
      ]),
      worktrees: [
        worktree('agent-1', 'stale', { item: 'I-1', linkSource: 'agent', branch: 'feat-1', dirty: 2, hygiene: { state: 'closing', why: 'its item is closed' } }),
        worktree('agent-2', 'active', { item: 'I-2' }),
      ],
    });
    await screen.findByText('Builds · 2');
    expect(container.querySelector('[data-launcher="web"]')).not.toBeNull();
    expect(container.querySelector('[data-launcher="other"]')).toBeNull();
    expect(container.querySelector('[data-launcher="master"]')).toBeNull();
    const closed = container.querySelector('[data-launcher="done"]') as HTMLElement;
    expect(within(closed).queryByRole('button', { name: /play/i })).toBeNull();
    const rows = within(screen.getByRole('list', { name: 'Worktrees' })).getAllByRole('listitem');
    expect(rows).toHaveLength(1);
    const line = rows[0] as HTMLElement;
    expect(line.textContent).toContain('agent-1');
    expect(line.textContent).toContain('feat-1');
    expect(within(line).getByText('closing')).toBeTruthy();
    expect(within(line).getByText('linked by agent')).toBeTruthy();
    expect(within(line).getByText('2 dirty')).toBeTruthy();
  });

  it('says so when nothing serves the item', async () => {
    await mountServed('I-1', withDefs([launcherDoc('master', { master: true })]));
    expect(await screen.findByText('No build serves this item.')).toBeTruthy();
    expect(screen.getByText('No worktree is linked to this item.')).toBeTruthy();
  });
});
