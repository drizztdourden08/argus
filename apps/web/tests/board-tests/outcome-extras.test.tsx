/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { doneDoc, testDoc } from './fixtures';
import type { Responder } from './mount-tab';
import { choose, openMenu } from './menu-dom';
import { mountTab } from './mount-tab';
import { attach, board, openBox, outcomeButton, png, refuseResults, send, type } from './outcome-dom';

cleanAfterEach();

const proofCalls = (calls: { url: string }[]) => calls.filter((call) => call.url.includes('/proofs'));

describe('S1 a screenshot with the note', { tags: ['standard'] }, () => {
  it('is chosen, shown with a remove button, and uploaded after the result posts with the note as caption', async () => {
    const { calls } = await mountTab(board());
    const box = await openBox('fail');
    type(box, 'the  sage is\ngone');
    attach(png('altar.png'));
    expect(screen.getByText('altar.png')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Remove the screenshot' })).toBeTruthy();
    send(box);
    await waitFor(() => expect(calls).toHaveLength(2));
    expect(calls[0]?.url).toBe('/api/p/rotp/tests/T-1/result');
    const upload = new URL(calls[1]?.url ?? '', 'http://dashboard');
    expect(upload.pathname).toBe('/api/p/rotp/tests/T-1/proofs');
    expect(upload.searchParams.get('caption')).toBe('the sage is gone');
    expect(calls[1]?.body).toBe('blob');
  });

  it('is taken away by the remove button', async () => {
    await mountTab(board());
    await openBox('fail');
    attach(png('altar.png'));
    fireEvent.click(screen.getByRole('button', { name: 'Remove the screenshot' }));
    expect(screen.queryByText('altar.png')).toBeNull();
  });

  it('stays in the box and uploads nothing when the send is refused', async () => {
    const { calls } = await mountTab(board(), refuseResults);
    const box = await openBox('fail');
    type(box, 'it broke');
    attach(png('altar.png'));
    send(box);
    expect(await screen.findByText('Refused: not your test.')).toBeTruthy();
    expect(proofCalls(calls)).toHaveLength(0);
    expect(screen.getByText('altar.png')).toBeTruthy();
  });

  it('is never taken when the file is not a picture', async () => {
    await mountTab(board());
    await openBox('fail');
    attach(new File(['x'], 'notes.txt', { type: 'text/plain' }));
    expect(screen.queryByText('notes.txt')).toBeNull();
  });

  it('is refused in a sentence when it is over 10 MB', async () => {
    await mountTab(board());
    await openBox('fail');
    const big = png('big.png');
    Object.defineProperty(big, 'size', { value: 11 * 1024 * 1024 });
    attach(big);
    expect(await screen.findByText('That image is over 10 MB.')).toBeTruthy();
    expect(screen.queryByText('big.png')).toBeNull();
  });
});

describe('S1 run again and the done row', { tags: ['standard'] }, () => {
  it('puts a done test back in To run with a rerun entry; a test still to run has no such button', async () => {
    const { calls } = await mountTab({ tests: [testDoc('T-1'), doneDoc('T-2', 'fail', 'no luck')] });
    expect(await waitFor(() => outcomeButton('pass', 'T-1'))).toBeTruthy();
    await openMenu('T-1');
    expect(screen.queryByRole('menuitem', { name: 'Run again' })).toBeNull();
    fireEvent.keyDown(screen.getByRole('menu'), { key: 'Escape' });
    await openMenu('T-2');
    await choose('Run again');
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/entries', method: 'POST', body: { kind: 'rerun', replyTo: 'T-2', author: 'human' } });
    expect((calls[0]?.body as { id: string }).id).toMatch(/^O-[a-z0-9]+-[a-z0-9]{4}$/);
  });

  it('leaves the button usable when the board refuses a rerun', async () => {
    const refuse: Responder = () => ({ status: 200, body: { id: 'O-1', status: 'refused', outcome: 'already pending', entry: null, seq: 1 } });
    const { calls } = await mountTab({ tests: [doneDoc('T-2', 'fail', 'no luck')] }, refuse);
    await openMenu('T-2');
    await choose('Run again');
    await waitFor(() => expect(calls).toHaveLength(1));
    await openMenu('T-2');
    expect((await screen.findByRole('menuitem', { name: 'Run again' })).getAttribute('aria-disabled')).not.toBe('true');
  });

  it('says who and when, shows the note, and hides the placeholder of a click without one', async () => {
    await mountTab({
      tests: [doneDoc('T-1', 'fail', 'the sage is gone'), doneDoc('T-2', 'pass', 'recorded on the board')],
    });
    expect(await screen.findByText('the sage is gone')).toBeTruthy();
    expect(screen.queryByText('recorded on the board')).toBeNull();
    expect(screen.getAllByText(/you ·/)).toHaveLength(2);
    expect(screen.queryByRole('group', { name: 'Run T-1' })).toBeNull();
  });
});
