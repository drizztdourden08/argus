/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { mountTab } from './mount-tab';
import { board, noteBox, openBox, resultCalls, send, sendButton, type } from './outcome-dom';

cleanAfterEach();

describe('the outcome buttons', () => {
  it('pass is one click: it posts at once with no note and opens nothing', async () => {
    const { calls } = await mountTab(board());
    fireEvent.click(await screen.findByRole('button', { name: 'Pass T-1' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/tests/T-1/result', method: 'POST', body: { outcome: 'pass', text: '' } });
    expect(screen.queryByRole('textbox', { name: 'Note for T-1' })).toBeNull();
  });

  it('fail opens the note box in the row: no post, a placeholder, Send fail disabled, the other buttons still there', async () => {
    const { calls } = await mountTab(board());
    const box = await openBox('Fail');
    expect(box.placeholder).toBe('What failed? The thread reads this.');
    expect(sendButton('fail').disabled).toBe(true);
    expect(calls).toHaveLength(0);
    for (const name of ['Pass T-1', 'Partial T-1', 'Blocked T-1']) expect(screen.getByRole('button', { name })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Fail T-1' }).getAttribute('aria-pressed')).toBe('true');
  });

  it('partial and blocked ask their own question', async () => {
    await mountTab(board());
    expect((await openBox('Partial')).placeholder).toBe('What works and what does not?');
    fireEvent.click(screen.getByRole('button', { name: 'Blocked T-1' }));
    expect(noteBox().placeholder).toBe('What blocks you?');
  });

  it('opens the box on one test only: the other row keeps just its buttons', async () => {
    await mountTab(board());
    await openBox('Fail');
    expect(screen.queryByRole('textbox', { name: 'Note for T-2' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Pass T-2' })).toBeTruthy();
  });

  it('a second click on pass while the first is posting does not post twice', async () => {
    const { calls } = await mountTab(board());
    const pass = await screen.findByRole('button', { name: 'Pass T-1' });
    fireEvent.click(pass);
    fireEvent.click(pass);
    await waitFor(() => expect(calls).toHaveLength(1));
  });
});

describe('the note box', () => {
  it('disables Send until the text has three non-space characters', async () => {
    await mountTab(board());
    const box = await openBox('Fail');
    type(box, 'a b');
    expect(sendButton('fail').disabled).toBe(true);
    type(box, 'a b c');
    expect(sendButton('fail').disabled).toBe(false);
    expect(noteBox().value).toBe('a b c');
  });

  it('never sends on Enter; Ctrl+Enter sends the trimmed text; too little text does not', async () => {
    const { calls } = await mountTab(board());
    const box = await openBox('Fail');
    type(box, '  it broke  ');
    fireEvent.keyDown(box, { key: 'Enter' });
    expect(calls).toHaveLength(0);
    type(box, 'ab');
    send(box);
    expect(calls).toHaveLength(0);
    type(box, '  it broke  ');
    send(box);
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect(resultCalls(calls)[0]?.body).toEqual({ outcome: 'fail', text: 'it broke' });
  });

  it('posts the same from the Send button and closes the box', async () => {
    const { calls } = await mountTab(board());
    type(await openBox('Partial'), 'works on pad one');
    fireEvent.click(sendButton('partial'));
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect(resultCalls(calls)[0]?.body).toEqual({ outcome: 'partial', text: 'works on pad one' });
    await waitFor(() => expect(screen.queryByRole('textbox', { name: 'Note for T-1' })).toBeNull());
  });

});

describe('closing and switching the note box', () => {
  it('closes on Esc and on Cancel, posts nothing and drops the text', async () => {
    const { calls } = await mountTab(board());
    const box = await openBox('Fail');
    type(box, 'half typed');
    fireEvent.keyDown(box, { key: 'Escape' });
    expect(screen.queryByRole('textbox', { name: 'Note for T-1' })).toBeNull();
    expect((await openBox('Fail')).value).toBe('');
    type(noteBox(), 'again');
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('textbox', { name: 'Note for T-1' })).toBeNull();
    expect(calls).toHaveLength(0);
  });

  it('posts pass at once and closes when pass is pressed while the box is open', async () => {
    const { calls } = await mountTab(board());
    type(await openBox('Fail'), 'something');
    fireEvent.click(screen.getByRole('button', { name: 'Pass T-1' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toEqual({ outcome: 'pass', text: '' });
    expect(screen.queryByRole('textbox', { name: 'Note for T-1' })).toBeNull();
  });

  it('keeps the text when another outcome of the same box is picked', async () => {
    await mountTab(board());
    type(await openBox('Fail'), 'the same words');
    fireEvent.click(screen.getByRole('button', { name: 'Partial T-1' }));
    expect(noteBox().value).toBe('the same words');
    expect(sendButton('partial')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Partial T-1' }).getAttribute('aria-pressed')).toBe('true');
    expect(screen.getByRole('button', { name: 'Fail T-1' }).getAttribute('aria-pressed')).toBe('false');
  });
});
