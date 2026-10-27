/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { leftover, metaDoc, oldMeta, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { cellOf, findRow, hasRowButton, rowNames, rowOf, text } from './wt-dom';

cleanAfterEach();

const LEFTOVER_TIP = 'git no longer lists this folder; only files are left';

describe('a leftover row', () => {
  it('says leftover, its size and file count, when it last changed, and its top entries; it can be deleted or asked about, not removed like a worktree', async () => {
    const row = leftover('agent-ab12');
    await mountTab({ docs: [row, worktree('feat', 'active', { ageDays: 0 })], meta: oldMeta() });
    const element = await findRow('agent-ab12');
    expect(within(element).getByTitle(LEFTOVER_TIP).textContent).toContain('leftover');
    expect(text(cellOf('agent-ab12', 'branch'))).toContain('1.5 GB');
    expect(text(cellOf('agent-ab12', 'branch'))).toContain('34210 files');
    expect(cellOf('agent-ab12', 'touched').querySelector('time')?.getAttribute('datetime')).toBe(row.leftover?.newestAt);
    expect(text(cellOf('agent-ab12', 'touched'))).toContain('last changed');
    expect(text(cellOf('agent-ab12', 'subject'))).toContain('dist/, logs/, notes.txt');
    expect(hasRowButton('agent-ab12', 'Delete leftover')).toBe(true);
    expect(hasRowButton('agent-ab12', 'Ask the thread to delete')).toBe(true);
    expect(hasRowButton('agent-ab12', 'Check')).toBe(true);
    expect(hasRowButton('agent-ab12', 'Remove')).toBe(false);
    expect(hasRowButton('agent-ab12', 'Delete now')).toBe(false);
  });

  it('posts nothing on its own, however many old leftovers there are', async () => {
    const { calls } = await mountTab({ docs: [leftover('a'), leftover('b')], meta: oldMeta() });
    await findRow('a');
    expect(calls).toEqual([]);
  });

  it('flags a nested repository, node_modules, a partial measure, a link and a folder not measured yet; a nested repository, a link and an unmeasured folder get no delete button', async () => {
    await mountTab({
      docs: [
        leftover('nested', {}, { hasNestedGit: true, nestedGit: 'inner/.git' }),
        leftover('nm', {}, { nodeModules: true }),
        leftover('big', {}, { partial: true }),
        leftover('lnk', {}, { isLink: true, sizeMB: 0, files: 0 }),
        leftover('new', {}, { pending: true, sizeMB: null, files: null, newestAt: null, ageDays: null }),
      ],
      meta: oldMeta(),
    });
    await findRow('nested');
    expect(within(rowOf('nested')).getByTitle(/inner\/\.git/).textContent).toBe('nested git');
    expect(hasRowButton('nested', 'Delete leftover')).toBe(false);
    expect(text(rowOf('nm'))).toContain('node_modules');
    expect(text(cellOf('nm', 'branch'))).toContain('≥ 1.5 GB');
    expect(text(rowOf('big'))).toContain('partial');
    expect(text(rowOf('lnk'))).toContain('link');
    expect(hasRowButton('lnk', 'Delete leftover')).toBe(false);
    expect(text(cellOf('new', 'branch'))).toContain('measuring');
    expect(hasRowButton('new', 'Delete leftover')).toBe(false);
    expect(hasRowButton('big', 'Delete leftover')).toBe(true);
  });
});

describe('escaping', () => {
  it('writes a hostile folder name and entry names as text', async () => {
    const evil = '<img src=x onerror=alert(1)>';
    await mountTab({ docs: [leftover('x', { path: `X:/p/${evil}`, name: evil }, { topEntries: [evil, '"><script>alert(2)</script>'], hasNestedGit: true, nestedGit: evil })], meta: oldMeta() });
    await findRow('x');
    expect(document.body.querySelector('img')).toBeNull();
    expect(document.body.querySelector('script')).toBeNull();
    expect(text(cellOf('x', 'subject'))).toContain(evil);
  });
});

describe('the summary and the Leftovers filter', () => {
  it('splits the git worktrees from the leftover folders and offers a Leftovers filter', async () => {
    const { container } = await mountTab({ docs: [leftover('a'), leftover('b'), worktree('feat', 'active', { ageDays: 0 })], meta: oldMeta() });
    await findRow('a');
    expect(text(container)).toContain('1 git worktree');
    expect(text(container)).toContain('2 leftover folders');
    fireEvent.click(screen.getByRole('radio', { name: 'Leftovers (2)' }));
    expect([...rowNames()].sort()).toEqual(['a', 'b']);
    expect(window.localStorage.getItem('coord.wtfilter')).toBe('left');
  });

  it('has no Leftovers filter when there are none', async () => {
    await mountTab({ docs: [worktree('feat', 'active')], meta: metaDoc() });
    await findRow('feat');
    expect(screen.queryByRole('radio', { name: /Leftovers/ })).toBeNull();
  });

  it('too many folders: delete the old leftovers', async () => {
    const { container } = await mountTab({ docs: [leftover('a'), leftover('b')], meta: oldMeta({ name: 'Bxel', tooMany: true }) });
    await findRow('a');
    expect(text(container)).toContain('Bxel has 2 folders: that is a lot, delete the old leftovers');
  });

  it('a saved Leftovers filter with none left says so', async () => {
    window.localStorage.setItem('coord.wtfilter', 'left');
    await mountTab({ docs: [worktree('feat', 'active')], meta: metaDoc() });
    expect(await screen.findByText('No leftover folders.')).toBeTruthy();
  });
});

describe('folders being deleted', () => {
  const going = () => [
    leftover('going', { status: 'deleting', job: { id: 'L-3', state: 'running', removed: 12, total: 40 } }),
    leftover('later', { status: 'deleting', job: { id: 'L-4', state: 'queued', removed: 0, total: null } }),
    leftover('stuck', { status: 'deleting', job: { id: 'L-5', state: 'failed', removed: 3, total: 9, error: 'build: EBUSY' } }),
  ];

  it('show the deleting status and the progress of the job, with no buttons to act on them', async () => {
    await mountTab({ docs: going(), meta: oldMeta() });
    await findRow('going');
    expect(within(rowOf('going')).getByTitle(/Argus is deleting this leftover folder in the background/).textContent).toContain('deleting');
    expect(text(cellOf('going', 'status'))).toContain('deleting, 12 of 40 entries');
    expect(text(cellOf('later', 'status'))).toContain('waiting for its turn');
    expect(text(cellOf('stuck', 'status'))).toContain('failed: build: EBUSY (finished at the next start)');
    for (const name of ['going', 'later', 'stuck']) {
      for (const label of ['Ask the thread to delete', 'Check', 'Delete leftover']) expect(hasRowButton(name, label)).toBe(false);
    }
  });

  it('a folder a batch guard refused is a banner that can be dismissed', async () => {
    const jobs = [{ id: 'L-9', name: 'young <b>one</b>', path: 'X:/p/young', state: 'refused', error: 'changed 2 hours ago, less than 3 days ago', finishedAt: new Date().toISOString() }];
    await mountTab({ docs: going(), meta: oldMeta({ jobs }) });
    expect(await screen.findByText('Not deleted: young <b>one</b> changed 2 hours ago, less than 3 days ago')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByText(/Not deleted:/)).toBeNull();
  });
});
