/* @layer renderer-app @kind test */
import { screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { testDoc } from './fixtures';
import { mountTab } from './mount-tab';
import { noteBox, openBox, resultCalls, send, sendButton, type } from './outcome-dom';

cleanAfterEach();

const LIMIT = 50000;
const SENTENCE = `This text is ${LIMIT + 1} characters and one post takes ${LIMIT} at most: shorten it or post it in two parts.`;
const board = () => ({ tests: [testDoc('T-1')] });

describe('S3 the length of a note', { tags: ['standard'] }, () => {
  it('a long note is posted whole: 20000 characters, no counter, no refusal', async () => {
    const { calls } = await mountTab(board());
    const long = 'line of feedback with **markdown** and a list\n- item\n'.repeat(400).slice(0, 20000);
    const box = await openBox('fail');
    type(box, long);
    expect(screen.queryByText(/characters/)).toBeNull();
    send(box);
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect(resultCalls(calls)[0]?.body).toEqual({ outcome: 'fail', text: long.trim() });
  });

  it('over 50000 characters the box says so, Send is off, nothing is posted or cut, and under the cap it goes', async () => {
    const { calls } = await mountTab(board());
    const box = await openBox('fail');
    type(box, 'x'.repeat(LIMIT + 1));
    expect(await screen.findByText(SENTENCE)).toBeTruthy();
    expect(sendButton('fail').disabled).toBe(true);
    send(box);
    await waitFor(() => expect(screen.getAllByText(SENTENCE).length).toBeGreaterThan(0));
    expect(resultCalls(calls)).toHaveLength(0);
    expect(noteBox().value).toHaveLength(LIMIT + 1);
    type(box, 'x'.repeat(LIMIT));
    expect(screen.queryAllByText(SENTENCE).length).toBeLessThanOrEqual(1);
    expect(sendButton('fail').disabled).toBe(false);
  });
});
