/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { metaDoc, rule, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import type { Responder } from './mount-tab';
import { findRow, messageBox, rowButton, text, typeMessage } from './wt-dom';

cleanAfterEach();

const board = () => ({ docs: [worktree('a', 'stale'), worktree('b', 'merged', { merged: true, unmerged: 0 })], meta: metaDoc() });

const askGroup = (verb: string, name: string): HTMLElement => screen.getByRole('group', { name: `Ask lead to ${verb} ${name}` });

const sendButton = (group: HTMLElement): HTMLButtonElement => within(group).getByRole<HTMLButtonElement>('button', { name: 'Send' });

const refused: Responder = () => ({ status: 200, body: { id: 'O-1', status: 'refused', outcome: 'the board is read-only right now', entry: null, seq: 1 } });

describe('opening the form', () => {
  it('opens a form under the row: who is asked, what the thread does, a message box, the key hint, Send and Cancel', async () => {
    await mountTab(board());
    await findRow('a');
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    const group = askGroup('delete', 'a');
    expect(text(group)).toContain('Ask lead to delete X:/proj/trees/a. The thread decides and answers here.');
    expect(messageBox().placeholder).toBe('A short message for the thread (optional): why it can go');
    expect(document.activeElement).toBe(messageBox());
    expect(text(group)).toContain('Ctrl+Enter sends. Enter adds a line.');
    expect(sendButton(group)).toBeTruthy();
    expect(within(group).getByRole('button', { name: 'Cancel' })).toBeTruthy();
    expect(rowButton('a', 'Ask the thread to delete').getAttribute('aria-expanded')).toBe('true');
    expect(screen.getAllByRole('group', { name: /^Ask lead to/ })).toHaveLength(1);
  });

  it('Check has its own wording; the button again closes the form; another row moves it', async () => {
    await mountTab(board());
    await findRow('a');
    fireEvent.click(rowButton('a', 'Check'));
    expect(text(askGroup('check on', 'a'))).toContain('Ask lead to check on X:/proj/trees/a.');
    expect(messageBox().placeholder).toBe('A short message for the thread (optional): what to check');
    fireEvent.click(rowButton('a', 'Check'));
    expect(screen.queryByRole('group', { name: /^Ask lead to/ })).toBeNull();
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    fireEvent.click(rowButton('b', 'Ask the thread to delete'));
    expect(screen.queryByRole('group', { name: 'Ask lead to delete a' })).toBeNull();
    expect(askGroup('delete', 'b')).toBeTruthy();
  });

  it('keeps the typed text when the same row switches between delete and check, not between rows', async () => {
    await mountTab(board());
    await findRow('a');
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    typeMessage('is the login work still here?');
    fireEvent.click(rowButton('a', 'Check'));
    expect(messageBox().value).toBe('is the login work still here?');
    fireEvent.click(rowButton('b', 'Check'));
    expect(messageBox().value).toBe('');
  });

  it('Cancel and Escape close it and post nothing', async () => {
    const { calls } = await mountTab(board());
    await findRow('a');
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    fireEvent.click(within(askGroup('delete', 'a')).getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('group', { name: /^Ask lead to/ })).toBeNull();
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    fireEvent.keyDown(askGroup('delete', 'a'), { key: 'Escape' });
    expect(screen.queryByRole('group', { name: /^Ask lead to/ })).toBeNull();
    expect(calls).toEqual([]);
  });
});

describe('sending', () => {
  it('Send posts a followup to the rule thread: the structured prefix with the path and the message, and the worktree field; then it closes', async () => {
    const { calls } = await mountTab(board());
    await findRow('a');
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    typeMessage('  it merged yesterday, it can go  ');
    fireEvent.click(sendButton(askGroup('delete', 'a')));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/alpha/entries', method: 'POST' });
    expect(calls[0]?.body).toMatchObject({
      kind: 'followup',
      author: 'human',
      thread: 'lead',
      replyTo: null,
      text: '[worktree delete] X:/proj/trees/a: it merged yesterday, it can go',
      worktree: { id: 'id-a', path: 'X:/proj/trees/a', kind: 'delete', message: 'it merged yesterday, it can go' },
    });
    expect(String((calls[0]?.body as { id: string }).id)).toMatch(/^O-[a-z0-9]+-[a-z0-9]{4}$/);
    await waitFor(() => expect(screen.queryByRole('group', { name: /^Ask lead to/ })).toBeNull());
  });

  it('a Check with no message posts the prefix alone', async () => {
    const { calls } = await mountTab(board());
    await findRow('b');
    fireEvent.click(rowButton('b', 'Check'));
    fireEvent.click(sendButton(askGroup('check on', 'b')));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({
      text: '[worktree check] X:/proj/trees/b:',
      worktree: { id: 'id-b', path: 'X:/proj/trees/b', kind: 'check', message: '' },
    });
  });

  it('Ctrl+Enter in the box sends; a plain Enter does not', async () => {
    const { calls } = await mountTab(board());
    await findRow('a');
    fireEvent.click(rowButton('a', 'Check'));
    typeMessage('hello');
    fireEvent.keyDown(messageBox(), { key: 'Enter' });
    expect(calls).toEqual([]);
    fireEvent.keyDown(messageBox(), { key: 'Enter', ctrlKey: true });
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({ text: '[worktree check] X:/proj/trees/a: hello' });
  });

  it('takes the thread from the rule when the row has none', async () => {
    const { calls } = await mountTab({ docs: [worktree('n', 'stale', { thread: undefined })], meta: metaDoc() });
    await findRow('n');
    fireEvent.click(rowButton('n', 'Check'));
    fireEvent.click(sendButton(askGroup('check on', 'n')));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({ thread: 'lead' });
  });
});

describe('when it goes wrong', () => {
  it('refuses a message over 600 characters with a sentence and keeps the form', async () => {
    const { calls } = await mountTab(board());
    await findRow('a');
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    typeMessage('x'.repeat(601));
    fireEvent.click(sendButton(askGroup('delete', 'a')));
    expect(text(askGroup('delete', 'a'))).toContain('The message is 601 characters; 600 at most.');
    expect(calls).toEqual([]);
  });

  it('a refusal by the board stays in the form with the text kept', async () => {
    const { calls } = await mountTab(board(), refused);
    await findRow('a');
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    typeMessage('try me');
    fireEvent.click(sendButton(askGroup('delete', 'a')));
    await waitFor(() => expect(text(askGroup('delete', 'a'))).toContain('Refused: the board is read-only right now.'));
    expect(messageBox().value).toBe('try me');
    expect(calls).toHaveLength(1);
  });

  it('a write that fails says so in the form', async () => {
    const failing: Responder = () => ({ status: 403, body: { error: { code: 'forbidden', message: 'no' } } });
    await mountTab(board(), failing);
    await findRow('a');
    fireEvent.click(rowButton('a', 'Check'));
    fireEvent.click(sendButton(askGroup('check on', 'a')));
    await waitFor(() => expect(text(askGroup('check on', 'a'))).toContain('Write refused: no'));
  });

  it('says there is no thread to tell when the rule names none, and posts nothing', async () => {
    const { calls } = await mountTab({ docs: [worktree('n', 'stale', { thread: '' })], meta: metaDoc({ rules: [rule({ thread: '' })] }) });
    await findRow('n');
    fireEvent.click(rowButton('n', 'Ask the thread to delete'));
    fireEvent.click(within(screen.getByRole('group', { name: /^Ask the thread to delete/ })).getByRole('button', { name: 'Send' }));
    expect(text(screen.getByRole('group', { name: /^Ask the thread to delete/ }))).toContain('No thread to tell: the rule of this worktree names none.');
    expect(calls).toEqual([]);
  });
});
