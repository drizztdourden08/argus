/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cardOf, cleanKanban, item, mountKanban } from './kanban-harness';

cleanKanban();

describe('I-98 a click on the card body opens it', { tags: ['standard'] }, () => {
  it('opens the item when the card surface is clicked', async () => {
    await mountKanban({ items: [item('I-1')] });
    await screen.findByText('Item number 1');
    fireEvent.click(cardOf('I-1'));
    expect(await screen.findByRole('dialog', { name: 'Work item' })).toBeTruthy();
  });
});
