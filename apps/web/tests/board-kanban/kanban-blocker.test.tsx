/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MIN, ago, boardOf, cleanKanban, item, mountKanban, openItem } from './kanban-harness';

cleanKanban();

describe('I-61 the blocker of an item', { tags: ['standard'] }, () => {
  it('shows what blocks the item as one lock chip with a count, never as a long tag, and a line in the drawer for the old text', async () => {
    const blocked = [{ kind: 'legacy', id: 'I-1', label: 'external: legal, the contract with a very long name that used to break the column' }];
    await mountKanban({ items: [item('I-1', { blocker: 'external: legal, the contract with a very long name that used to break the column', blocked }), item('I-2')] });
    const board = within(await boardOf());
    expect(board.getAllByTitle(/^Blocked by external: legal/)).toHaveLength(1);
    expect(board.queryByText(/^blocked: /)).toBeNull();
    const dialog = await openItem('I-1');
    expect(within(dialog).getByText(/^blocked: external: legal/)).toBeTruthy();
  });

  it('counts several causes in one chip and names them in the tooltip', async () => {
    const blocked = [
      { kind: 'item', id: 'I-9', label: 'Ship the lock' },
      { kind: 'todo', id: 'H-12', label: 'push go', who: 'owner' },
      { kind: 'test', id: 'T-3', label: 'Login works' },
    ];
    await mountKanban({ items: [item('I-1', { blocked })] });
    const chip = within(await boardOf()).getByTitle('Blocked by I-9: Ship the lock; H-12 (owner): push go; T-3: Login works');
    expect(chip.textContent).toBe('3');
  });

  it('shows a blocker that names an open question as the chip of that question, in the card and in the drawer', async () => {
    const question = { id: 'Q-1', kind: 'question', item: 'I-2', author: 'worker', status: 'open', text: 'Which one?', createdAt: ago(MIN) };
    const blocked = [{ kind: 'legacy', id: 'I-1', label: 'Q-1' }, { kind: 'ask', id: 'Q-1', label: 'Which one?' }];
    await mountKanban({ items: [item('I-1', { blocker: 'Q-1', blocked }), item('I-2')], entries: [question] });
    const board = within(await boardOf());
    expect(board.getAllByTitle('Q-1 is an open question, waiting for an answer')).toHaveLength(2);
    expect(board.queryByTitle(/^Blocked by/)).toBeNull();
    const dialog = await openItem('I-1');
    expect(within(dialog).getByText('blocked on')).toBeTruthy();
    expect(within(dialog).getByRole('link', { name: 'Q-1' })).toBeTruthy();
  });

  it('shows no blocker on an item without one', async () => {
    await mountKanban({ items: [item('I-1', { blocker: null })] });
    await screen.findByText('Item I-1');
    expect(screen.queryByTitle(/^Blocked by/)).toBeNull();
  });
});
