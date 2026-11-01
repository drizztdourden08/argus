/* @layer renderer-app @kind test */
import { within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HOUR, MIN, NOW, ago, metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { findRow, titleOf } from './wt-dom';

cleanAfterEach();

const docs = () => [
  worktree('a', 'stale', { hygiene: { state: 'released', agent: 'A-3', thread: 'lead', since: ago(MIN), goesAt: new Date(NOW + HOUR).toISOString() } }),
  worktree('b', 'stale', { hygiene: { state: 'review', agent: 'A-4', thread: 'lead', why: 'it has 1 uncommitted change' } }),
  worktree('c', 'stale', { hygiene: { state: 'kept', agent: 'A-5', thread: 'lead', why: 'still testing' } }),
  worktree('d', 'stale'),
  worktree('e', 'active', { hygiene: { state: 'in-use', agent: 'A-6', thread: 'lead', why: 'its agent A-6 is working' } }),
  worktree('f', 'active', { main: true }),
];

describe('S1 the worktree hygiene on the Worktrees tab', { tags: ['standard'] }, () => {
  it('tags each worktree in use, released, review or kept with the reason, and leaves the others alone', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    const released = within(await findRow('a')).getByText(/^released, goes /);
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
});
