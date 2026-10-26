/* @layer renderer-app @kind test */
import { screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import type { Responder } from './mount-tab';
import { mountTab } from './mount-tab';
import { board, noteBox, openBox, refuseResults, resultCalls, send, sendButton, type } from './outcome-dom';

cleanAfterEach();

const failWith = async (respond: Responder | undefined, text = 'it broke') => {
  const mounted = await mountTab(board(), respond);
  const box = await openBox('Fail');
  type(box, text);
  send(box);
  return mounted;
};

describe('a refusal of the board', () => {
  it('keeps the box and its text and says why', async () => {
    const { calls } = await failWith(refuseResults);
    expect(await screen.findByText('Refused: not your test.')).toBeTruthy();
    expect(noteBox().value).toBe('it broke');
    expect(resultCalls(calls)).toHaveLength(1);
  });

  it('says on a conflict that nothing of yours was applied', async () => {
    await failWith(() => ({ status: 200, body: { id: 'O-1', status: 'conflict', outcome: null, entry: null, seq: 1 } }));
    expect(await screen.findByText(/Nothing of yours was applied/)).toBeTruthy();
  });

  it('says an unreachable dashboard is retrying and keeps the box', async () => {
    await failWith(() => ({ status: 503, body: { error: { code: 'unavailable', message: 'down' } } }));
    expect(await screen.findByText('Dashboard unreachable, retrying.')).toBeTruthy();
    expect(noteBox().value).toBe('it broke');
  });
});

describe('a long note', () => {
  it('is posted whole, with no limit below 50000 characters', async () => {
    const long = 'word '.repeat(4000).trim();
    const { calls } = await failWith(undefined, long);
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect((resultCalls(calls)[0]?.body as { text: string }).text).toBe(long);
  });

  it('over 50000 characters is said in a sentence under the box and in a toast, Send is off and nothing is posted', async () => {
    const { calls } = await failWith(undefined, 'x'.repeat(50001));
    expect(await screen.findAllByText(/50001 characters and one post takes 50000 at most/)).toHaveLength(2);
    expect(sendButton('fail').disabled).toBe(true);
    expect(calls).toHaveLength(0);
  });
});
