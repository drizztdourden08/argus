/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { metaDoc, rule, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { findRow, text } from './wt-dom';

cleanAfterEach();

const docs = () => [worktree('stale-1', 'stale'), worktree('active-1', 'active', { ageDays: 0 })];

describe('the rules', () => {
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

  it('shows a wildcard rule as invalid and ignored, once', async () => {
    window.localStorage.setItem('coord.wtrules', '1');
    await mountTab({ docs: docs(), meta: metaDoc({ rules: rules() }) });
    await findRow('stale-1');
    const body = text(document.body.querySelector('details') as HTMLElement);
    expect(body.split('invalid: wildcard path')).toHaveLength(2);
    const chip = screen.getByTitle(/A rule path is the parent folder, never a wildcard/);
    expect(chip.textContent).toBe('invalid: wildcard path');
  });
});

describe('the empty states', () => {
  it('no rule yet: the thread has not said where its worktrees live, and how it does', async () => {
    const { container } = await mountTab({ docs: [], meta: metaDoc({ rules: [] }) });
    expect(await screen.findByText('The thread has not told Argus where its worktrees live yet.')).toBeTruthy();
    expect(text(container)).toContain('coord worktree rule add <path>');
    expect(document.body.querySelector('[role="grid"]')).toBeNull();
  });

  it('no summary yet is the same empty state', async () => {
    await mountTab({ docs: [], meta: null });
    expect(await screen.findByText('The thread has not told Argus where its worktrees live yet.')).toBeTruthy();
  });

  it('rules but nothing found; before the first scan it says the scan has not finished', async () => {
    await mountTab({ docs: [], meta: metaDoc() });
    expect(await screen.findByText('No worktrees found under the rules.')).toBeTruthy();
    expect(screen.queryByText('The first scan has not finished yet.')).toBeNull();
  });

  it('the first scan has not finished', async () => {
    const { container } = await mountTab({ docs: [], meta: metaDoc({ scannedAt: null }) });
    expect(await screen.findByText('The first scan has not finished yet.')).toBeTruthy();
    expect(text(container)).toContain('not scanned yet');
  });
});
