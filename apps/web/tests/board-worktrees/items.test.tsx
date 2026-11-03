/* @layer renderer-app @kind test */
import { within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HOUR, NOW, metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { cellOf, findRow, text, titleOf } from './wt-dom';

cleanAfterEach();

const docs = () => [
  worktree('linked', 'active', { item: 'I-12', linkSource: 'agent' }),
  worktree('loose', 'active', { unlinked: true }),
  worktree('main-1', 'active', { main: true }),
  worktree('ending', 'stale', { item: 'I-7', linkSource: 'branch', hygiene: { state: 'closing', why: 'its item is closed', goesAt: new Date(NOW + HOUR).toISOString() } }),
];

describe('P13 the Item column of the Worktrees tab', { tags: ['standard'] }, () => {
  it('links the item, marks an unlinked worktree, and leaves a main checkout blank', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('linked');
    const linked = within(cellOf('linked', 'item')).getByText('I-12');
    expect(linked.closest('[href]')?.getAttribute('href')).toBe('#I-12');
    expect(titleOf(linked)).toContain('linked by agent');
    expect(text(cellOf('loose', 'item'))).toBe('no item');
    expect(text(cellOf('main-1', 'item'))).toBe('');
  });

  it('shows the closing state with the time it goes and why', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('ending');
    const closing = within(cellOf('ending', 'status')).getByText(/^closing, goes /);
    expect(titleOf(closing)).toContain('Its item closed');
    expect(text(cellOf('ending', 'item'))).toBe('I-7');
  });
});
