/* @layer renderer-app @kind test */
import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import type { Responder } from './mount-tab';
import { mountTab } from './mount-tab';
import { board, noteBox, openBox, refuseResults, resultCalls, send, type } from './outcome-dom';

cleanAfterEach();

const failWith = async (respond: Responder) => {
  const mounted = await mountTab(board(), respond);
  const box = await openBox('fail');
  type(box, 'it broke');
  send(box);
  return mounted;
};

describe('S3 a refusal of the board', { tags: ['standard'] }, () => {
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
