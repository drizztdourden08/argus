/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { opened, postedEdit } from './kanban-drawer-dom';
import { cleanKanban } from './kanban-harness';

cleanKanban();

describe('the edits of the owner in the drawer: title and subject', () => {
  it('edits the title with an edit op that carries the old title', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit title' }));
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'title' }), { target: { value: 'Sign in' } });
    fireEvent.keyDown(within(dialog).getByRole('textbox', { name: 'title' }), { key: 'Enter' });
    await postedEdit(posted, { title: 'Sign in' }, { title: 'Login' });
    expect(await screen.findByText('Saved I-1.')).toBeTruthy();
  });

  it('does not take an empty title or one over 80 characters', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit title' }));
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'title' }), { target: { value: '   ' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save title' }));
    expect(within(dialog).getByRole('alert').textContent).toBe('That cannot be empty.');
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'title' }), { target: { value: 'x'.repeat(81) } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save title' }));
    expect(within(dialog).getByRole('alert').textContent).toBe('At most 80 characters.');
    expect(posted).toHaveLength(0);
  });

  it('edits the subject, and may empty it', async () => {
    const { posted, dialog } = await opened({ subject: 'ui' });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit subject' }));
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'subject' }), { target: { value: '' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save subject' }));
    await postedEdit(posted, { subject: '' }, { subject: 'ui' });
  });
});

describe('the edits of the owner in the drawer: description', () => {
  it('edits the description in the Markdown editor and saves it with Ctrl+Enter', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit description' }));
    const box = within(dialog).getByRole('textbox', { name: 'Description' });
    fireEvent.change(box, { target: { value: 'New **text**' } });
    fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true });
    await postedEdit(posted, { description: 'New **text**' }, { description: 'Make **login** work. See [the spec](https://example.com/spec).\n\n- one\n- two' });
  });

  it('refuses a description over 400 characters in a plain sentence and keeps the editor open', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit description' }));
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Description' }), { target: { value: 'x'.repeat(401) } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save description' }));
    expect(within(dialog).getByText('At most 400 characters.')).toBeTruthy();
    expect(within(dialog).getByRole('textbox', { name: 'Description' })).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('cancels the description edit with the button, leaving the text', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit description' }));
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Description' }), { target: { value: 'gone' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    expect(within(dialog).getByText('login')).toBeTruthy();
    expect(posted).toHaveLength(0);
  });
});
