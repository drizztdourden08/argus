/* @layer renderer-app @kind test */
import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { WorktreeRequest } from '../../src/data/worktrees';
import { ago, metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { findRow, text } from './wt-dom';

cleanAfterEach();

const request = (extra: Partial<WorktreeRequest> = {}): WorktreeRequest => ({
  id: 'R-3',
  by: 'lead',
  at: ago(5 * 60_000),
  mode: 'finished',
  status: 'partial',
  branch: false,
  total: 3,
  removed: 2,
  refused: 1,
  skipped: 0,
  note: null,
  sentences: ['X:/p/w/a has 1 uncommitted change'],
  ...extra,
});

const docs = () => [worktree('a', 'stale')];

describe('S1 the removal requests of threads (read-only)', { tags: ['standard'] }, () => {
  it('say the branches and the skipped ones, and show only the newest eight', async () => {
    const many = Array.from({ length: 10 }, (_, n) => request({ id: `R-${10 - n}`, status: 'done', sentences: [] }));
    many[0] = request({ id: 'R-10', status: 'done', mode: 'leftovers', branch: true, skipped: 4, refused: 0, note: 'no leftover folder **old** enough' });
    await mountTab({ docs: docs(), meta: metaDoc({ requests: many }) });
    await findRow('a');
    const body = text(screen.getByRole('group', { name: 'Removal requests from threads' }));
    expect(body).toContain('every old leftover folder and merged branches');
    expect(body).toContain('2 removed, 4 skipped · no leftover folder old enough');
    expect(body).toContain('R-3');
    expect(body).not.toContain('R-2');
  });
});

describe('K4 the removal requests of threads', { tags: ['standard'] }, () => {
  it('keep a hostile sentence and name as plain text', async () => {
    const hostile = request({ by: '<b>x</b>', sentences: ['<img src=x onerror=alert(1)>'] });
    await mountTab({ docs: docs(), meta: metaDoc({ requests: [hostile] }) });
    await findRow('a');
    const group = screen.getByRole('group', { name: 'Removal requests from threads' });
    expect(group.querySelector('img')).toBeNull();
    expect(group.querySelectorAll('b')).toHaveLength(1);
  });
});
