/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DAY, HOUR, ago, metaDoc, rule, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { cellOf, findRow, hasRowButton, rowButton, rowNames, rowOf, text } from './wt-dom';

cleanAfterEach();

const STALE_TIP = 'Not touched for the stale limit or more, and not merged. The thread was told.';

const docs = () => [
  worktree('active-1', 'active', { lastTouchedAt: ago(2 * HOUR), ageDays: 0 }),
  worktree('stale-1', 'stale', { ahead: 2, behind: 1 }),
  worktree('merged-1', 'merged', { ageDays: 9, merged: true, unmerged: 0 }),
  worktree('dirty-1', 'dirty-stale', { dirty: 3, ageDays: 5 }),
  worktree('plain-1', 'not-git', { branch: null, head: null, subject: null, lastTouchedAt: null, ageDays: null }),
  worktree('kept-1', 'kept', { kept: { until: ago(-5 * DAY), note: 'login work', by: 'lead' } }),
];

describe('the table', () => {
  it('lists what needs a decision first, the oldest first, then the rest', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('stale-1');
    expect(rowNames()).toEqual(['merged-1', 'stale-1', 'dirty-1', 'kept-1', 'active-1', 'plain-1']);
  });

  it('a row shows its path with the rule and thread, the status with its tip, branch and commit, touched, ahead and behind, dirty and last commit', async () => {
    const rows = docs();
    await mountTab({ docs: rows, meta: metaDoc() });
    const row = await findRow('stale-1');
    expect(text(cellOf('stale-1', 'path'))).toContain('X:/proj/trees/stale-1');
    expect(text(cellOf('stale-1', 'path'))).toContain('rule W-1, tells lead');
    expect(within(row).getByTitle(STALE_TIP).textContent).toContain('stale');
    expect(text(cellOf('stale-1', 'branch'))).toContain('abc1234');
    expect(text(cellOf('stale-1', 'ahead'))).toBe('2 ahead, 1 behind');
    expect(text(cellOf('stale-1', 'subject'))).toContain('work on stale-1');
    const touched = cellOf('stale-1', 'touched').querySelector('time');
    expect(touched?.getAttribute('datetime')).toBe(rows[1]?.lastTouchedAt);
    expect(text(cellOf('dirty-1', 'dirty'))).toBe('3');
    expect(within(rowOf('merged-1')).getByTitle(/can go\. The thread was told/).textContent).toContain('merged, can go');
    expect(within(rowOf('dirty-1')).getByTitle(/never "can go"/).textContent).toContain('dirty, stale');
    expect(within(rowOf('plain-1')).getByTitle('A folder under the rule that is not a git work tree.').textContent).toContain('not git');
  });

  it('a kept worktree says until when, and why', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('kept-1');
    const tip = within(rowOf('kept-1')).getByTitle(/no stale signal until the date/);
    expect(tip.getAttribute('title')).toMatch(/Until \d{4}-\d\d-\d\d: login work \(lead\)\./);
  });

  it('marks a main checkout, a locked worktree and a detached HEAD', async () => {
    await mountTab({
      docs: [worktree('m', 'stale', { main: true }), worktree('l', 'stale', { locked: 'demo' }), worktree('d', 'stale', { detached: true, branch: null })],
      meta: metaDoc(),
    });
    await findRow('m');
    expect(within(rowOf('m')).getByTitle("The repository's own checkout, not a linked worktree").textContent).toBe('main');
    expect(within(rowOf('l')).getByTitle('Locked: demo').textContent).toBe('locked');
    expect(within(rowOf('d')).getByTitle('HEAD is detached').textContent).toBe('detached');
  });
});

describe('the row actions and escaping', () => {
  it('every row can ask the thread to delete or check; Remove only when clean, Delete now for lost work, nothing on a main, locked, not-git or error row', async () => {
    await mountTab({
      docs: [
        ...docs(),
        worktree('loose-1', 'stale', { detached: true, branch: null, referenced: false, unmerged: 2 }),
        worktree('main-1', 'stale', { main: true }),
        worktree('locked-1', 'stale', { locked: 'demo' }),
        worktree('err-1', 'error'),
      ],
      meta: metaDoc(),
    });
    await findRow('stale-1');
    expect(rowButton('stale-1', 'Ask the thread to delete').getAttribute('aria-expanded')).toBe('false');
    expect(hasRowButton('stale-1', 'Check')).toBe(true);
    expect(hasRowButton('stale-1', 'Remove')).toBe(true);
    expect(hasRowButton('merged-1', 'Remove')).toBe(true);
    expect(hasRowButton('dirty-1', 'Delete now')).toBe(true);
    expect(hasRowButton('loose-1', 'Delete now')).toBe(true);
    for (const name of ['main-1', 'locked-1', 'err-1', 'plain-1']) {
      expect(hasRowButton(name, 'Remove')).toBe(false);
      expect(hasRowButton(name, 'Delete now')).toBe(false);
    }
  });

  it('posts nothing on its own: showing rows that can go changes nothing', async () => {
    const { calls } = await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('merged-1');
    expect(calls).toEqual([]);
  });

  it('writes a hostile path, branch, subject and request as text, never as markup', async () => {
    const evil = '<img src=x onerror=alert(1)>';
    const ask = { id: 'O-1', kind: 'check' as const, text: evil, at: ago(60_000), by: 'owner', answer: { note: evil, at: ago(1000), by: evil } };
    await mountTab({
      docs: [worktree('x', 'stale', { path: `X:/p/${evil}`, branch: `"><script>alert(2)</script>`, subject: evil, rule: evil, thread: evil, request: ask })],
      meta: metaDoc({ rules: [rule({ path: evil, thread: evil, note: evil, error: evil, exists: false })] }),
    });
    await findRow('x');
    expect(document.body.querySelector('img')).toBeNull();
    expect(document.body.querySelector('script')).toBeNull();
    expect(text(cellOf('x', 'path'))).toContain(evil);
    expect(text(cellOf('x', 'subject'))).toContain(evil);
  });
});

describe('the request on a row', () => {
  const asked = { id: 'O-1', kind: 'delete' as const, text: 'it merged **yesterday**', at: ago(5 * 60_000), by: 'owner', answer: null };
  const answered = { ...asked, id: 'O-2', kind: 'check' as const, text: 'still needed?', answer: { note: 'kept for the login work', at: ago(60_000), by: 'lead' } };

  it('pending: a chip with the kind and the time, the text with its marks drawn', async () => {
    await mountTab({ docs: [worktree('a', 'stale', { request: asked })], meta: metaDoc() });
    await findRow('a');
    const cell = cellOf('a', 'request');
    expect(within(cell).getByTitle('Waiting for the thread to answer (coord worktree done)').textContent).toBe('asked to delete, waiting');
    expect(cell.querySelector('strong')?.textContent).toBe('yesterday');
    expect(cell.querySelector('time')).not.toBeNull();
  });

  it('answered: the thread, when it answered, and its note', async () => {
    await mountTab({ docs: [worktree('b', 'merged', { request: answered })], meta: metaDoc() });
    await findRow('b');
    const cell = cellOf('b', 'request');
    expect(within(cell).getByTitle('Asked to check').textContent).toBe('answered');
    expect(text(cell)).toContain('lead');
    expect(text(cell)).toContain('still needed?');
    expect(text(cell)).toContain('kept for the login work');
    expect(text(cell)).not.toContain('waiting');
  });
});

describe('the summary and the filter', () => {
  it('counts the worktrees, the ones that need a decision, the scan, and the rule problems', async () => {
    const meta = metaDoc({ rules: [rule(), rule({ id: 'W-2', exists: false }), rule({ id: 'W-3', error: 'not a git repository' })], gone: 1 });
    const { container } = await mountTab({ docs: docs(), meta });
    await findRow('stale-1');
    const line = text(container);
    expect(line).toContain('6 worktrees');
    expect(line).toContain('3 need a decision');
    expect(line).toContain('scanned');
    expect(line).toContain('1 gone (git still lists it)');
    expect(line).toContain('2 rule problems');
  });

  it('says so when there are too many worktrees', async () => {
    const many = Array.from({ length: 14 }, (_, index) => worktree(`w${index}`, 'active', { ageDays: 0 }));
    const { container } = await mountTab({ docs: many, meta: metaDoc({ tooMany: true }) });
    await findRow('w0');
    expect(text(container)).toContain('Alpha has 14 worktrees: that is a lot, the thread should clean up');
  });

  it('filters to the ones that need a decision, and remembers the choice', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('stale-1');
    expect(screen.getByRole('radio', { name: 'All (6)' }).getAttribute('aria-checked')).toBe('true');
    fireEvent.click(screen.getByRole('radio', { name: 'Need a decision (3)' }));
    expect(rowNames()).toEqual(['merged-1', 'stale-1', 'dirty-1']);
    expect(window.localStorage.getItem('coord.wtfilter')).toBe('need');
    fireEvent.click(screen.getByRole('radio', { name: 'All (6)' }));
    expect(rowNames()).toHaveLength(6);
  });

  it('applies a saved filter on load; when nothing needs a decision it says so and offers all', async () => {
    window.localStorage.setItem('coord.wtfilter', 'need');
    await mountTab({ docs: [worktree('a', 'active')], meta: metaDoc() });
    expect(await screen.findByText('Nothing needs a decision.')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Show all 1' }));
    expect(rowNames()).toEqual(['a']);
  });
});
