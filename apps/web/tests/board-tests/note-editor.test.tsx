/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { mountTab } from './mount-tab';
import { board, noteBox, openBox, resultCalls, sendButton, type } from './outcome-dom';

cleanAfterEach();

describe('S1 the note box of a result', { tags: ['standard'] }, () => {
  it('takes the focus when it opens, with the caret at the end', async () => {
    await mountTab(board());
    const box = await openBox('fail');
    expect(document.activeElement).toBe(box);
  });
});

describe('S1 the keys and buttons of the note box', { tags: ['standard'] }, () => {
  it('a toolbar button formats the selection, and the draft survives with Send following the text', async () => {
    await mountTab(board());
    const box = await openBox('fail');
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
    const box = await openBox('fail');
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
});
