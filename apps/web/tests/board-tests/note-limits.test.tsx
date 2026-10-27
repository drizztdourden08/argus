/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, testDoc } from './fixtures';
import { openFrom } from './menu-dom';
import { mountTab } from './mount-tab';
import { noteBox, openBox, resultCalls, send, sendButton, type } from './outcome-dom';

cleanAfterEach();

const LIMIT = 50000;
const SENTENCE = `This text is ${LIMIT + 1} characters and one post takes ${LIMIT} at most: shorten it or post it in two parts.`;
const PROOFS = [
  { n: 1, caption: 'the first view', url: '/files/rotp/proofs/T-1-1.png', at: '2026-10-26T09:00:00.000Z' },
  { n: 2, caption: 'the second view', url: '/files/rotp/proofs/T-1-2.png', at: '2026-10-26T09:00:00.000Z' },
];

const withProofs = () => ({ builds: [buildDoc('bundle-1')], tests: [testDoc('T-1', { proofs: PROOFS })] });

describe('the length of a note', () => {
  it('a long note is posted whole: 20000 characters, no counter, no refusal', async () => {
    const { calls } = await mountTab(withProofs());
    const long = 'line of feedback with **markdown** and a list\n- item\n'.repeat(400).slice(0, 20000);
    const box = await openBox('fail');
    type(box, long);
    expect(screen.queryByText(/characters/)).toBeNull();
    send(box);
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect(resultCalls(calls)[0]?.body).toEqual({ outcome: 'fail', text: long.trim() });
  });

  it('over 50000 characters the box says so, Send is off, nothing is posted or cut, and under the cap it goes', async () => {
    const { calls } = await mountTab(withProofs());
    const box = await openBox('fail');
    type(box, 'x'.repeat(LIMIT + 1));
    expect(await screen.findByText(SENTENCE)).toBeTruthy();
    expect(sendButton('fail').disabled).toBe(true);
    send(box);
    await waitFor(() => expect(screen.getAllByText(SENTENCE)).toHaveLength(2));
    expect(resultCalls(calls)).toHaveLength(0);
    expect(noteBox().value).toHaveLength(LIMIT + 1);
    type(box, 'x'.repeat(LIMIT));
    expect(screen.getAllByText(SENTENCE)).toHaveLength(1);
    expect(sendButton('fail').disabled).toBe(false);
  });
});

describe('the note dialog with screenshots', () => {
  it('shows the screenshots already attached, with their captions', async () => {
    await mountTab(withProofs());
    const dialog = within(await openFrom('T-1', 'Add a note...'));
    expect(dialog.getByAltText('the first view').getAttribute('src')).toBe('/files/rotp/proofs/T-1-1.png');
    expect(dialog.getByAltText('the second view').getAttribute('src')).toBe('/files/rotp/proofs/T-1-2.png');
    expect(dialog.getAllByRole('button', { name: /^Open the / })).toHaveLength(2);
    fireEvent.click(dialog.getByRole('button', { name: 'Cancel' }));
  });
});
