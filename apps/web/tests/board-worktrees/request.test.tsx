/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import type { Responder } from './mount-tab';
import { chooseAction, messageBox, text, typeMessage } from './wt-dom';

cleanAfterEach();

const ASK_DELETE = /^Ask the thread to delete/;
const CHECK = /^Check/;

const board = () => ({ docs: [worktree('a', 'stale'), worktree('b', 'merged', { merged: true, unmerged: 0 })], meta: metaDoc() });

const askGroup = (verb: string, name: string): HTMLElement => screen.getByRole('group', { name: `Ask lead to ${verb} ${name}` });

const sendButton = (group: HTMLElement): HTMLButtonElement => within(group).getByRole<HTMLButtonElement>('button', { name: 'Send' });

const refused: Responder = () => ({ status: 200, body: { id: 'O-1', status: 'refused', outcome: 'the board is read-only right now', entry: null, seq: 1 } });

describe('I-85 sending a request to the thread from the row menu', { tags: ['standard'] }, () => {
  it('Send posts a followup to the rule thread: the structured prefix with the path and the message, and the worktree field; then it closes', async () => {
    const { calls } = await mountTab(board());
    await chooseAction('a', ASK_DELETE);
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

  it('Ctrl+Enter in the box sends; a plain Enter does not', async () => {
    const { calls } = await mountTab(board());
    await chooseAction('a', CHECK);
    typeMessage('hello');
    fireEvent.keyDown(messageBox(), { key: 'Enter' });
    expect(calls).toEqual([]);
    fireEvent.keyDown(messageBox(), { key: 'Enter', ctrlKey: true });
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({ text: '[worktree check] X:/proj/trees/a: hello' });
  });
});

describe('I-85 when sending a request goes wrong', { tags: ['standard'] }, () => {
  it('refuses a message over 600 characters with a sentence and keeps the form', async () => {
    const { calls } = await mountTab(board());
    await chooseAction('a', ASK_DELETE);
    typeMessage('x'.repeat(601));
    fireEvent.click(sendButton(askGroup('delete', 'a')));
    expect(text(askGroup('delete', 'a'))).toContain('The message is 601 characters; 600 at most.');
    expect(calls).toEqual([]);
  });

  it('a refusal by the board stays in the form with the text kept', async () => {
    const { calls } = await mountTab(board(), refused);
    await chooseAction('a', ASK_DELETE);
    typeMessage('try me');
    fireEvent.click(sendButton(askGroup('delete', 'a')));
    await waitFor(() => expect(text(askGroup('delete', 'a'))).toContain('Refused: the board is read-only right now.'));
    expect(messageBox().value).toBe('try me');
    expect(calls).toHaveLength(1);
  });

  it('a write that fails says so in the form', async () => {
    const failing: Responder = () => ({ status: 403, body: { error: { code: 'forbidden', message: 'no' } } });
    await mountTab(board(), failing);
    await chooseAction('a', CHECK);
    fireEvent.click(sendButton(askGroup('check on', 'a')));
    await waitFor(() => expect(text(askGroup('check on', 'a'))).toContain('Write refused: no'));
  });
});
