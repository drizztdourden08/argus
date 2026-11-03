/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanEntryBoard, entryWork, mountEntryBoard } from '../helpers/entry-board';

cleanEntryBoard();

const recordDialog = () => screen.queryByRole('dialog', { name: 'Work item', hidden: true });

describe('I-85 the record pane follows its address', { tags: ['standard'] }, () => {
  it('leaves no record dialog in the page once the address no longer names a record', async () => {
    await mountEntryBoard({ at: '/p/rotp/?open=I-1#agents', work: entryWork(), entries: {} });
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(recordDialog()).toBe(dialog);
    fireEvent.keyDown(dialog, { key: 'Escape' });
    await waitFor(() => expect(recordDialog()).toBeNull());
  });

  it('leaves no record dialog when the scrim is clicked', async () => {
    await mountEntryBoard({ at: '/p/rotp/?open=I-1#kanban', work: entryWork(), entries: {} });
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    fireEvent.click(dialog.parentElement?.querySelector('.drawer__scrim') as HTMLElement);
    await waitFor(() => expect(recordDialog()).toBeNull());
  });
});
