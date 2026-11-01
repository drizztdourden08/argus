/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HOUR, MIN, ago, metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { findRow, text } from './wt-dom';

cleanAfterEach();

const inAnHour = new Date(Date.now() + HOUR).toISOString();

const docs = () => [
  worktree('a', 'stale', { hygiene: { state: 'released', agent: 'A-3', thread: 'lead', since: ago(MIN), goesAt: inAnHour } }),
  worktree('b', 'stale', { hygiene: { state: 'review', agent: 'A-4', thread: 'lead', why: 'it has 1 uncommitted change' } }),
  worktree('c', 'stale', { hygiene: { state: 'kept', agent: 'A-5', thread: 'lead', why: 'still testing' } }),
  worktree('d', 'stale'),
];

const removed = [{ id: 'e1', path: 'X:/proj/trees/e', branch: 'e', thread: 'lead', at: ago(5 * MIN), text: 'removed worktree X:/proj/trees/e, branch e deleted' }];

describe('the worktree hygiene on the Worktrees tab', () => {
  it('tags each ended agent row released (with the time it goes), in review or kept, and leaves the others alone', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    const a = await findRow('a');
    const released = within(a).getByText(/^released, goes /);
    expect(released.getAttribute('title')).toContain('Its agent A-3 ended and nothing uses it: Argus removes it at ');
    expect(released.getAttribute('title')).toContain('unless lead keeps it (coord worktree keep) or releases it now (coord worktree release).');
    const review = within(await findRow('b')).getByText('in review');
    expect(review.getAttribute('title')).toBe('Its agent A-4 ended, but Argus does not remove it by itself: it has 1 uncommitted change.');
    const kept = within(await findRow('c')).getByText('kept');
    expect(kept.getAttribute('title')).toBe('Its agent A-5 ended; the thread keeps it: still testing.');
    const d = await findRow('d');
    expect(within(d).queryByText(/released|in review/)).toBeNull();
  });

  it('lists the recent removals under the table, read-only', async () => {
    await mountTab({ docs: docs(), meta: metaDoc({ hygiene: { on: true, graceMs: HOUR, removed } }) });
    await findRow('a');
    const group = screen.getByRole('group', { name: 'Worktrees Argus removed by itself' });
    const body = text(group);
    expect(body).toContain('Removed by the worktree hygiene: the worktrees of ended sub-agents, newest first.');
    expect(body).toMatch(/removedX:\/proj\/trees\/elead · .* ago/);
    expect(within(group).getByText('removed').getAttribute('title')).toBe('removed worktree X:/proj/trees/e, branch e deleted');
    expect(within(group).queryAllByRole('button')).toHaveLength(0);
  });
});
