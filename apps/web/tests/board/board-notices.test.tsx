/* @layer renderer-app @kind test */
import { within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { passiveNotices } from '../../src/screens/board/BoardHeader/behavior/passive-notices';
import { minutesAgo as at } from '../helpers/minutes-ago';
import { openAlerts } from '../helpers/open-alerts';
import { openRotp } from '../helpers/open-rotp';
import { stubCss } from '../helpers/stub-css';

stubCss();

const lead = { name: 'lead', role: 'lead', status: 'active', activity: 'idle', session: 'RotP', lastSeen: at(0) };

const WAKES = {
  'W-1': { thread: 'lead', reason: 'kanban-stale', text: 'I-4 sits in doing for 3 days', status: 'passive', listeners: 0, at: at(30) },
  'W-2': { thread: 'lead', reason: 'test-fail', text: 'T-1 failed', status: 'delivered', listeners: 1, at: at(20) },
  'W-3': { thread: 'ui', reason: 'worktree-review', text: '3 worktrees need a decision', status: 'passive', listeners: 0, at: at(10) },
};

const open = (wakes: Record<string, object>) => openRotp({ wakes, threads: { lead } });

describe('K1 the board notices', { tags: ['critical'] }, () => {
  it('keeps only the passive rows, newest first', () => {
    const docs = Object.entries(WAKES).map(([id, data]) => ({ id, ...data }));
    expect(passiveNotices(docs).map((doc) => doc.id)).toEqual(['W-3', 'W-1']);
  });
});

describe('S1 the board notices in the Alerts popover', { tags: ['standard'] }, () => {
  it('counts the notices on the Alerts button and lists them under Notices on click', async () => {
    await open(WAKES);
    const { button, panel } = await openAlerts();
    expect(within(button).getByText('2')).toBeTruthy();
    expect(within(panel).getByText('Notices (2)')).toBeTruthy();
    const list = within(panel).getByRole('list', { name: 'Notices' });
    const rows = within(list).getAllByRole('listitem');
    expect(rows).toHaveLength(2);
    expect(rows[0]?.textContent).toContain('worktree-review');
    expect(rows[0]?.textContent).toContain('3 worktrees need a decision');
    expect(rows[1]?.textContent).toContain('I-4 sits in doing for 3 days');
    expect(within(panel).queryByText('T-1 failed')).toBeNull();
  });
});
