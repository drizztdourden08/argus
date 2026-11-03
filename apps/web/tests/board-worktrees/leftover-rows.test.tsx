/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ago, leftover, oldMeta } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { findRow, hasAction, openMenu, rowOf, text } from './wt-dom';

cleanAfterEach();

const DELETE_LEFTOVER = /^Delete leftover/;

describe('I-85 a leftover row', { tags: ['standard'] }, () => {
  it('flags a nested repository, a link and a folder not measured yet, and gives those no delete action', async () => {
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
    expect(text(rowOf('nested'))).toContain('nested git');
    expect(text(rowOf('nm'))).toContain('node_modules');
    expect(text(rowOf('nm'))).toContain('≥ 1.5 GB');
    expect(text(rowOf('big'))).toContain('partial');
    expect(text(rowOf('lnk'))).toContain('link');
    expect(text(rowOf('new'))).toContain('measuring');
    for (const name of ['nested', 'lnk', 'new']) {
      await openMenu(name);
      expect(hasAction(DELETE_LEFTOVER)).toBe(false);
      fireEvent.keyDown(document.body, { key: 'Escape' });
    }
    await openMenu('big');
    expect(hasAction(DELETE_LEFTOVER)).toBe(true);
  });

  it('writes a hostile folder name and entry names as text', async () => {
    const evil = '<img src=x onerror=alert(1)>';
    await mountTab({ docs: [leftover('x', { path: `X:/p/${evil}`, name: evil }, { topEntries: [evil, '"><script>alert(2)</script>'], hasNestedGit: true, nestedGit: evil })], meta: oldMeta() });
    await findRow('x');
    expect(document.body.querySelector('img')).toBeNull();
    expect(document.body.querySelector('script')).toBeNull();
    expect(text(rowOf('x'))).toContain(evil);
  });
});

describe('I-85 folders being deleted', { tags: ['standard'] }, () => {
  const going = () => [
    leftover('going', { status: 'deleting', job: { id: 'L-3', state: 'running', removed: 12, total: 40 } }),
    leftover('later', { status: 'deleting', job: { id: 'L-4', state: 'queued', removed: 0, total: null } }),
    leftover('stuck', { status: 'deleting', job: { id: 'L-5', state: 'failed', removed: 3, total: 9, error: 'build: EBUSY' } }),
  ];

  it('show the progress of the job, with no actions to ask, check or delete them', async () => {
    await mountTab({ docs: going(), meta: oldMeta() });
    await findRow('going');
    expect(text(rowOf('going'))).toContain('deleting, 12 of 40 entries');
    expect(text(rowOf('later'))).toContain('waiting for its turn');
    expect(text(rowOf('stuck'))).toContain('failed: build: EBUSY (finished at the next start)');
    for (const name of ['going', 'later', 'stuck']) {
      await openMenu(name);
      for (const label of [/^Ask the thread to delete/, /^Check/, DELETE_LEFTOVER]) expect(hasAction(label)).toBe(false);
      fireEvent.keyDown(document.body, { key: 'Escape' });
    }
  });

  it('a folder a batch guard refused is a banner that can be dismissed', async () => {
    const jobs = [{ id: 'L-9', name: 'young <b>one</b>', path: 'X:/p/young', state: 'refused', error: 'changed 2 hours ago, less than 3 days ago', finishedAt: ago(60_000) }];
    await mountTab({ docs: going(), meta: oldMeta({ jobs }) });
    expect(await screen.findByText('Not deleted: young <b>one</b> changed 2 hours ago, less than 3 days ago')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByText(/Not deleted:/)).toBeNull();
  });
});
