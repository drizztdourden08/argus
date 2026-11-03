/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MIN, ago, boardOf, cleanKanban, item, mountKanban, openItem } from './kanban-harness';

cleanKanban();

describe('I-61 the blocker of an item', { tags: ['standard'] }, () => {
  it('shows the blocker as a tag on the card and a line in the drawer', async () => {
    await mountKanban({ items: [item('I-1', { blocker: 'D-11' }), item('I-2')] });
    expect(within(await boardOf()).getAllByText('blocked: D-11')).toHaveLength(1);
    const dialog = await openItem('I-1');
    expect(within(dialog).getByText('blocked: D-11')).toBeTruthy();
  });

  it('shows a blocker that names an open question as the tag of that question, in the card and in the drawer', async () => {
    const question = { id: 'Q-1', kind: 'question', item: 'I-2', author: 'worker', status: 'open', text: 'Which one?', createdAt: ago(MIN) };
    await mountKanban({ items: [item('I-1', { blocker: 'Q-1' }), item('I-2')], entries: [question] });
    const board = await boardOf();
    expect(within(board).getAllByText('waiting on Q-1')).toHaveLength(2);
    expect(within(board).queryByText('blocked: Q-1')).toBeNull();
    const dialog = await openItem('I-1');
    expect(within(dialog).getByText('blocked on')).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Open Q-1' })).toBeTruthy();
  });

  it('shows no blocker text on an item without one', async () => {
    await mountKanban({ items: [item('I-1', { blocker: null })] });
    await screen.findByText('Item I-1');
    expect(screen.queryByText(/^blocked: /)).toBeNull();
  });
});
