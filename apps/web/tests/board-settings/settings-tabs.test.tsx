/* @layer renderer-app @kind test */
import { fireEvent, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanKanban, mountKanban, openSettings } from '../board-kanban/kanban-harness';

cleanKanban();

describe('I-221 the board settings dialog', { tags: ['critical'] }, () => {
  it('shows three tabs and no brand mark choice', async () => {
    await mountKanban();
    const dialog = await openSettings();
    const names = within(dialog).getAllByRole('tab').map((tab) => tab.textContent);
    expect(names).toEqual(['Appearance', 'General', 'Kanban']);
    expect(within(dialog).queryByText(/brand mark/i)).toBeNull();
  });

  it('keeps each concept on its own tab', async () => {
    await mountKanban();
    const dialog = await openSettings();
    expect(within(dialog).getByText('Board icon')).toBeTruthy();
    expect(within(dialog).queryByRole('spinbutton', { name: 'Title length limit' })).toBeNull();
    fireEvent.click(within(dialog).getByRole('tab', { name: 'General' }));
    expect(within(dialog).getByRole('spinbutton', { name: 'Title length limit' })).toBeTruthy();
    fireEvent.click(within(dialog).getByRole('tab', { name: 'Kanban' }));
    expect(within(dialog).getByRole('button', { name: 'Add a column' })).toBeTruthy();
  });
});
