/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DAY, HOUR, ago, metaDoc, rule, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { cellOf, findRow, hasRowButton, rowButton, rowNames, text } from './wt-dom';

cleanAfterEach();

const docs = () => [
  worktree('active-1', 'active', { lastTouchedAt: ago(2 * HOUR), ageDays: 0 }),
  worktree('stale-1', 'stale', { ahead: 2, behind: 1 }),
  worktree('merged-1', 'merged', { ageDays: 9, merged: true, unmerged: 0 }),
  worktree('dirty-1', 'dirty-stale', { dirty: 3, ageDays: 5 }),
  worktree('plain-1', 'not-git', { branch: null, head: null, subject: null, lastTouchedAt: null, ageDays: null }),
  worktree('kept-1', 'kept', { kept: { until: ago(-5 * DAY), note: 'login work', by: 'lead' } }),
];

describe('K4 the row actions', { tags: ['standard'] }, () => {
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

describe('S1 the summary and the filter', { tags: ['standard'] }, () => {
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
