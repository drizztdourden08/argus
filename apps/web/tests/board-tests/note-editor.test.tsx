/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { openFrom } from './menu-dom';
import { mountTab } from './mount-tab';
import { board, noteBox, openBox, resultCalls, send, sendButton, type } from './outcome-dom';

cleanAfterEach();

const TOOLS = ['Bold', 'Italic', 'Bulleted list', 'Numbered list', 'Quote', 'Link', 'Inline code', 'Code block'];

const toolbarOf = (scope: HTMLElement): string[] =>
  within(within(scope).getByRole('toolbar', { name: 'Formatting' })).getAllByRole('button').map((button) => button.getAttribute('aria-label') ?? '');

describe('the note box of a result', () => {
  it('is a small markdown editor: a toolbar of eight named buttons, three lines, a hint, one limit', async () => {
    await mountTab(board());
    const box = await openBox('Fail');
    const group = screen.getByRole('group', { name: 'fail note for T-1' });
    expect(toolbarOf(group)).toEqual(TOOLS);
    expect(box.rows).toBe(3);
    expect(box.maxLength).toBe(50000);
    expect(within(group).getByText('Ctrl+Enter sends')).toBeTruthy();
    expect(box.placeholder).toBe('What failed? The thread reads this.');
  });

  it('takes the focus when it opens, with the caret at the end', async () => {
    await mountTab(board());
    const box = await openBox('Fail');
    expect(document.activeElement).toBe(box);
  });

  it('asks a different question for partial and blocked in the same box', async () => {
    await mountTab(board());
    expect((await openBox('Partial')).placeholder).toBe('What works and what does not?');
    expect((await openBox('Blocked')).placeholder).toBe('What blocks you?');
    expect(screen.getAllByRole('textbox', { name: 'Note for T-1' })).toHaveLength(1);
  });

  it('is the same editor in the note dialog and in the drawer of a test', async () => {
    await mountTab(board());
    const dialog = await openFrom('T-1', 'Add a note...');
    expect(toolbarOf(dialog)).toEqual(TOOLS);
    const area = within(dialog).getByRole<HTMLTextAreaElement>('textbox', { name: 'Result note for T-1' });
    expect([area.rows, area.maxLength]).toEqual([3, 50000]);
    expect(document.activeElement).toBe(area);
    fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: /Add a note/ })).toBeNull());
    fireEvent.click(await screen.findByRole('button', { name: 'Test T-1' }));
    const drawer = await screen.findByRole('dialog');
    fireEvent.click(within(drawer).getByRole('button', { name: 'Fail T-1' }));
    expect(toolbarOf(within(drawer).getByRole('group', { name: 'fail note for T-1' }))).toEqual(TOOLS);
  });

});

describe('the keys and buttons of the note box', () => {
  it('a toolbar button formats the selection, and the draft survives with Send following the text', async () => {
    await mountTab(board());
    const box = await openBox('Fail');
    type(box, 'say hi now');
    box.setSelectionRange(4, 6);
    fireEvent.click(screen.getByRole('button', { name: 'Bold' }));
    expect(box.value).toBe('say **hi** now');
    box.setSelectionRange(0, 0);
    fireEvent.click(screen.getByRole('button', { name: 'Bulleted list' }));
    expect(box.value).toBe('- say **hi** now');
    expect(noteBox().value).toBe('- say **hi** now');
    expect(sendButton('fail').disabled).toBe(false);
  });

  it('Ctrl+B and Ctrl+I wrap the selection, and Cmd+Enter sends the trimmed text', async () => {
    const { calls } = await mountTab(board());
    const box = await openBox('Fail');
    type(box, 'one two');
    box.setSelectionRange(0, 3);
    fireEvent.keyDown(box, { key: 'b', ctrlKey: true });
    expect(box.value).toBe('**one** two');
    type(box, 'say hi');
    box.setSelectionRange(4, 6);
    fireEvent.keyDown(box, { key: 'i', metaKey: true });
    expect(box.value).toBe('say *hi*');
    fireEvent.keyDown(box, { key: 'Enter', metaKey: true });
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect(resultCalls(calls)[0]?.body).toEqual({ outcome: 'fail', text: 'say *hi*' });
  });

  it('Enter alone sends nothing, Ctrl+Enter does', async () => {
    const { calls } = await mountTab(board());
    const box = await openBox('Fail');
    type(box, 'The button does nothing');
    fireEvent.keyDown(box, { key: 'Enter' });
    expect(resultCalls(calls)).toHaveLength(0);
    send(box);
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
  });

  it('Escape leaves the box without a post', async () => {
    const { calls } = await mountTab(board());
    const box = await openBox('Fail');
    type(box, 'keep this out');
    fireEvent.keyDown(box, { key: 'Escape' });
    expect(screen.queryByRole('textbox', { name: 'Note for T-1' })).toBeNull();
    expect(resultCalls(calls)).toHaveLength(0);
  });
});
