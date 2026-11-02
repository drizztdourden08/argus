/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { boardOf, cleanKanban, item, mountKanban, openItem } from './kanban-harness';

cleanKanban();

describe('I-61 the blocker of an item', { tags: ['standard'] }, () => {
  it('shows the blocker as a tag on the card and a line in the drawer', async () => {
    await mountKanban({ items: [item('I-1', { blocker: 'D-11' }), item('I-2')] });
    expect(within(await boardOf()).getAllByText('blocked: D-11')).toHaveLength(1);
    const dialog = await openItem('I-1');
    expect(within(dialog).getByText('blocked: D-11')).toBeTruthy();
  });

  it('shows no blocker text on an item without one', async () => {
    await mountKanban({ items: [item('I-1', { blocker: null })] });
    await screen.findByText('Item I-1');
    expect(screen.queryByText(/^blocked: /)).toBeNull();
  });
});
