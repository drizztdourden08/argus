/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, doneDoc, itemDoc, testDoc } from './fixtures';
import { entries, firstResult, openFrom } from './menu-dom';
import { mountTab } from './mount-tab';
import { resultCalls } from './outcome-dom';

cleanAfterEach();

const board = () => ({
  builds: [buildDoc('bundle-1')],
  items: [itemDoc('I-1'), itemDoc('I-2')],
  tests: [testDoc('T-1', { rev: 4, covers: 'AC1' }), doneDoc('T-2', 'fail', 'broken')],
});

describe('S1 the note dialog of the row menu', { tags: ['standard'] }, () => {
  it('takes any outcome, but a fail without text is not sent', async () => {
    const { calls } = await mountTab(board());
    const dialog = within(await openFrom('T-1', 'Add a note...'));
    expect(dialog.getByText('Add a note to T-1')).toBeTruthy();
    fireEvent.change(dialog.getByRole('textbox', { name: 'Result note for T-1' }), { target: { value: '   ' } });
    fireEvent.click(dialog.getByRole('button', { name: 'Post T-1 as fail' }));
    expect(await dialog.findByText('Say what you saw: three characters at least.')).toBeTruthy();
    expect(resultCalls(calls)).toHaveLength(0);
    fireEvent.click(dialog.getByRole('button', { name: 'Post T-1 as pass' }));
    expect(await firstResult(calls)).toEqual({ outcome: 'pass', text: '' });
  });

  it('posts a note with the outcome picked and closes', async () => {
    const { calls } = await mountTab(board());
    const dialog = within(await openFrom('T-1', 'Add a note...'));
    fireEvent.change(dialog.getByRole('textbox', { name: 'Result note for T-1' }), { target: { value: 'works on the second try' } });
    fireEvent.click(dialog.getByRole('button', { name: 'Post T-1 as partial' }));
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect(resultCalls(calls)[0]?.body).toEqual({ outcome: 'partial', text: 'works on the second try' });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('adds a note to an existing result with Ctrl+Enter, as the result it already has', async () => {
    const { calls } = await mountTab(board());
    const dialog = within(await openFrom('T-2', 'Add a note...'));
    const box = dialog.getByRole('textbox', { name: 'Result note for T-2' });
    fireEvent.change(box, { target: { value: 'still broken on Windows' } });
    fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true });
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect(resultCalls(calls)[0]).toMatchObject({ url: '/api/p/rotp/tests/T-2/result', body: { outcome: 'fail', text: 'still broken on Windows' } });
  });

  it('cancels without a post', async () => {
    const { calls } = await mountTab(board());
    const dialog = within(await openFrom('T-1', 'Add a note...'));
    fireEvent.click(dialog.getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(calls).toHaveLength(0);
  });
});

describe('S1 the edit dialog of the row menu', { tags: ['standard'] }, () => {
  it('saves the changed fields in one edit with what the owner saw', async () => {
    const { calls } = await mountTab(board());
    const dialog = within(await openFrom('T-1', 'Edit'));
    fireEvent.change(dialog.getByRole('textbox', { name: 'Title' }), { target: { value: '  A better title ' } });
    fireEvent.change(dialog.getByRole('textbox', { name: 'Build' }), { target: { value: 'bundle-9' } });
    fireEvent.click(dialog.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(entries(calls)).toHaveLength(1));
    expect(entries(calls)[0]?.body).toMatchObject({
      kind: 'edit',
      target: { c: 'tests', id: 'T-1' },
      set: { title: 'A better title', build: 'bundle-9' },
      prev: { title: 'Test T-1', build: 'bundle-1' },
      baseRev: 4,
    });
  });

  it('refuses an empty title and posts nothing', async () => {
    const { calls } = await mountTab(board());
    const dialog = within(await openFrom('T-1', 'Edit'));
    fireEvent.change(dialog.getByRole('textbox', { name: 'Title' }), { target: { value: ' ' } });
    fireEvent.click(dialog.getByRole('button', { name: 'Save' }));
    expect(await dialog.findByText('The title cannot be empty.')).toBeTruthy();
    expect(calls).toHaveLength(0);
  });

  it('posts nothing when nothing changed', async () => {
    const { calls } = await mountTab(board());
    const dialog = within(await openFrom('T-1', 'Edit'));
    fireEvent.click(dialog.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(calls).toHaveLength(0);
  });
});
