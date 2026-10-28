/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { WorktreeRequest } from '../../src/data/worktrees';
import { metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { findRow, text } from './wt-dom';

cleanAfterEach();

const request = (extra: Partial<WorktreeRequest> = {}): WorktreeRequest => ({
  id: 'R-3',
  by: 'lead',
  at: new Date(Date.now() - 5 * 60_000).toISOString(),
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

describe('the removal requests of threads (read-only)', () => {
  it('list each request with a status chip, who asked, what, and how it went; nothing to click', async () => {
    const requests = [
      request(),
      request({ id: 'R-2', status: 'queued', mode: 'targets', total: 2, sentences: [] }),
      request({ id: 'R-1', status: 'done', refused: 0, removed: 3, sentences: [] }),
    ];
    await mountTab({ docs: docs(), meta: metaDoc({ requests }) });
    await findRow('a');
    const group = screen.getByRole('group', { name: 'Removal requests from threads' });
    const body = text(group);
    expect(body).toContain('Removal requests from threads. Argus removes with the same guards as the Delete button; this list is read-only.');
    expect(body).toMatch(/R-3partiallead asked to remove every finished worktree · .* ago · 2 removed, 1 refusedX:\/p\/w\/a has 1 uncommitted change/);
    expect(body).toMatch(/R-2queuedlead asked to remove 2 worktrees · .* ago(?! · )/);
    expect(body).toMatch(/R-1donelead asked to remove every finished worktree · .* ago · 3 removed/);
    expect(within(group).getByTitle('Some were refused by the guards').textContent).toBe('partial');
    expect(within(group).getByTitle('Waiting for Argus').textContent).toBe('queued');
    expect(within(group).getByTitle('Every target was handled').textContent).toBe('done');
    expect(within(group).queryAllByRole('button')).toHaveLength(0);
  });

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

  it('show no block without requests', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('a');
    expect(screen.queryByRole('group', { name: 'Removal requests from threads' })).toBeNull();
  });

  it('keep a hostile sentence and name as plain text', async () => {
    const hostile = request({ by: '<b>x</b>', sentences: ['<img src=x onerror=alert(1)>'] });
    await mountTab({ docs: docs(), meta: metaDoc({ requests: [hostile] }) });
    await findRow('a');
    const group = screen.getByRole('group', { name: 'Removal requests from threads' });
    expect(group.querySelector('img')).toBeNull();
    expect(group.querySelectorAll('b')).toHaveLength(1);
  });
});
