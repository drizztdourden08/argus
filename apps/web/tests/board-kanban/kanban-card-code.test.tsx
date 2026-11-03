/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { boardOf, cardOf, cleanKanban, item, mountKanban, paneOf } from './kanban-harness';

cleanKanban();

const typed = {
  items: [item('I-1', { type: 'bug' }), item('I-2', { type: 'epic' }), item('I-3', { type: 'story', parent: 'I-2' }), item('I-4')],
};

describe('I-85 the code of a card is the code of its type', { tags: ['critical'] }, () => {
  it('shows the letters of the type on the head, with the number the item is stored under', async () => {
    await mountKanban(typed);
    await boardOf();
    for (const [id, code] of [['I-1', 'B-1'], ['I-2', 'E-2'], ['I-3', 'S-3'], ['I-4', 'K-4']] as const) {
      expect(within(cardOf(id)).getByRole('link', { name: code })).toBeTruthy();
      expect(within(cardOf(id)).getByRole('button', { name: `More actions for ${code}` })).toBeTruthy();
    }
  });

  it('names the card to open by its typed code', async () => {
    await mountKanban(typed);
    expect(await screen.findByRole('button', { name: 'Open B-1: Item I-1' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Open E-2: Item I-2' })).toBeTruthy();
  });

  it('opens the pane from the typed code of the head', async () => {
    await mountKanban(typed);
    await boardOf();
    fireEvent.click(within(cardOf('I-1')).getByRole('link', { name: 'B-1' }));
    await waitFor(() => expect(paneOf().open).toBe('B-1'));
    expect(await screen.findByRole('dialog', { name: 'Work item' })).toBeTruthy();
  });
});

describe('I-85 the pane accepts the stored id and the typed code, and only the right letters', { tags: ['critical'] }, () => {
  it('opens the item named by its old stored id', async () => {
    await mountKanban(typed, undefined, '#I-1');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(within(dialog).getByRole('link', { name: 'B-1' })).toBeTruthy();
  });

  it('opens the item named by the code of its type', async () => {
    await mountKanban(typed, undefined, '#B-1');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(within(dialog).getByRole('link', { name: 'B-1' })).toBeTruthy();
  });

  it('does not open an item under the letters of another type', async () => {
    await mountKanban(typed, undefined, '#S-1');
    expect(await screen.findByText('S-1 is not on the board alpha.')).toBeTruthy();
  });
});
