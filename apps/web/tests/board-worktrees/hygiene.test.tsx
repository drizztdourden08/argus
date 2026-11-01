/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HOUR, MIN, ago, metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { findRow, text, titleOf } from './wt-dom';

cleanAfterEach();

const inAnHour = new Date(Date.now() + HOUR).toISOString();

const docs = () => [
  worktree('a', 'stale', { hygiene: { state: 'released', agent: 'A-3', thread: 'lead', since: ago(MIN), goesAt: inAnHour } }),
  worktree('b', 'stale', { hygiene: { state: 'review', agent: 'A-4', thread: 'lead', why: 'it has 1 uncommitted change' } }),
  worktree('c', 'stale', { hygiene: { state: 'kept', agent: 'A-5', thread: 'lead', why: 'still testing' } }),
  worktree('d', 'stale'),
  worktree('e', 'active', { hygiene: { state: 'in-use', agent: 'A-6', thread: 'lead', why: 'its agent A-6 is working' } }),
  worktree('f', 'active', { main: true }),
];

const removed = [{ id: 'e1', path: 'X:/proj/trees/e', branch: 'e', thread: 'lead', at: ago(5 * MIN), text: 'removed worktree X:/proj/trees/e, branch e deleted' }];

describe('the worktree hygiene on the Worktrees tab', () => {
  it('tags each worktree in use, released (with the time it goes), review or kept, and leaves the others alone', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    const a = await findRow('a');
    const released = within(a).getByText(/^released, goes /);
    expect(titleOf(released)).toContain('No running agent owns it, nothing uses it and removing it loses nothing: Argus removes it at ');
    expect(titleOf(released)).toContain('unless lead keeps it (coord worktree keep) or releases it now (coord worktree release).');
    const review = within(await findRow('b')).getByText('review');
    expect(titleOf(review)).toBe('Argus does not remove it by itself: it has 1 uncommitted change.');
    const kept = within(await findRow('c')).getByText('kept');
    expect(titleOf(kept)).toBe('The thread keeps it: still testing.');
    const inUse = within(await findRow('e')).getByText('in use');
    expect(titleOf(inUse)).toBe('In use, so Argus leaves it: its agent A-6 is working.');
    const d = await findRow('d');
    expect(within(d).queryByText(/released|review|in use/)).toBeNull();
  });

  it('never shows the three day "active" state: the hygiene state is the state', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('a');
    expect(screen.queryByText('active')).toBeNull();
  });

  it('keeps the staleness tag only on a row in review or without a hygiene state', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    expect(within(await findRow('a')).queryByText('stale')).toBeNull();
    expect(within(await findRow('c')).queryByText('stale')).toBeNull();
    expect(within(await findRow('b')).getByText('stale')).toBeTruthy();
    expect(within(await findRow('d')).getByText('stale')).toBeTruthy();
  });

  it('lists the recent removals under the table, read-only', async () => {
    await mountTab({ docs: docs(), meta: metaDoc({ hygiene: { on: true, graceMs: HOUR, removed } }) });
    await findRow('a');
    const group = screen.getByRole('group', { name: 'Worktrees Argus removed by itself' });
    const body = text(group);
    expect(body).toContain('Removed by the worktree hygiene: the released worktrees, newest first.');
    expect(body).toMatch(/removedX:\/proj\/trees\/elead · .* ago/);
    expect(titleOf(within(group).getByText('removed'))).toBe('removed worktree X:/proj/trees/e, branch e deleted');
    expect(within(group).queryAllByRole('button')).toHaveLength(0);
  });
});
