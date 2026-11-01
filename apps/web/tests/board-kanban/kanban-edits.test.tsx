/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { opened, postedEdit } from './kanban-drawer-dom';
import { cleanKanban } from './kanban-harness';

cleanKanban();

describe('K3 the edits of the owner in the drawer', { tags: ['standard'] }, () => {
  it('edits the title with an edit op that carries the old title', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit title: Login' }));
    const box = within(dialog).getByRole('textbox', { name: 'Edit title' });
    fireEvent.change(box, { target: { value: 'Sign in' } });
    fireEvent.keyDown(box, { key: 'Enter' });
    await postedEdit(posted, { title: 'Sign in' }, { title: 'Login' });
    expect(await screen.findByText('Saved I-1.')).toBeTruthy();
  });

  it('does not take an empty title or one over 80 characters', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit title: Login' }));
    const box = within(dialog).getByRole('textbox', { name: 'Edit title' });
    fireEvent.change(box, { target: { value: '   ' } });
    fireEvent.keyDown(box, { key: 'Enter' });
    expect(within(dialog).getByText('That cannot be empty.')).toBeTruthy();
    fireEvent.change(box, { target: { value: 'x'.repeat(81) } });
    fireEvent.keyDown(box, { key: 'Enter' });
    expect(within(dialog).getByText('At most 80 characters.')).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('edits the description in the Markdown editor and saves it with Ctrl+Enter', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit description' }));
    const box = within(dialog).getByRole('textbox', { name: 'Description' });
    fireEvent.change(box, { target: { value: 'New **text**' } });
    fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true });
    await postedEdit(posted, { description: 'New **text**' }, { description: 'Make **login** work. See [the spec](https://example.com/spec).\n\n- one\n- two' });
  });
});
