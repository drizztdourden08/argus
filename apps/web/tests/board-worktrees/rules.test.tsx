/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { metaDoc, rule, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { findRow, text } from './wt-dom';

cleanAfterEach();

const docs = () => [worktree('stale-1', 'stale'), worktree('active-1', 'active', { ageDays: 0 })];

describe('S1 the rules', { tags: ['standard'] }, () => {
  const rules = () => [
    rule({ include: ['web-*'] }),
    rule({ id: 'W-2', exclude: ['web-*', 'x?'], thread: 'worker', staleDays: 7, note: 'the rest', candidates: 1 }),
    rule({ id: 'W-3', path: 'X:/rotp/trees/web-*', invalid: 'wildcard path', exists: null, error: 'invalid: wildcard path', candidates: null }),
    rule({ id: 'W-4', exists: false, error: 'not a git repository' }),
  ];

  it('are folded by default; opening them lists each rule read-only and remembers the choice', async () => {
    await mountTab({ docs: docs(), meta: metaDoc({ rules: rules() }) });
    await findRow('stale-1');
    const fold = document.body.querySelector('details');
    expect(fold?.open).toBe(false);
    fireEvent.click(screen.getByText('Rules (4)'));
    expect(document.body.querySelector('details')?.open).toBe(true);
    expect(window.localStorage.getItem('coord.wtrules')).toBe('1');
    const body = text(document.body.querySelector('details') as HTMLElement);
    expect(body).toContain('The registered thread adds these with coord worktree rule add.');
    expect(body).toContain('W-1');
    expect(body).toContain('include web-*');
    expect(body).toContain('exclude web-* x?');
    expect(body).toContain('tells worker, stale after 7 days, 1 folder');
    expect(body).toContain('the rest');
    expect(body).toContain('not found');
    expect(body).toContain('not a git repository');
  });
});
