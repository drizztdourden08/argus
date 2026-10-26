/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, doneDoc, testDoc } from './fixtures';
import { firstResult } from './menu-dom';
import { mountTab } from './mount-tab';
import { board, noteBox, openBox, refuseResults, resultCalls, send, type } from './outcome-dom';

cleanAfterEach();

const rowButton = async (id = 'T-1'): Promise<HTMLElement> => screen.findByRole('button', { name: `Test ${id}` });

describe('the keys 1 to 4 on a test to run', () => {
  it('say what they do above the list', async () => {
    await mountTab(board());
    expect(await screen.findByText('keys 1 pass, 2 fail, 3 partial, 4 blocked (2 to 4 ask what you saw first)')).toBeTruthy();
  });

  it('1 posts pass at once with no note', async () => {
    const { calls } = await mountTab(board());
    fireEvent.keyDown(await rowButton(), { key: '1' });
    expect(await firstResult(calls)).toEqual({ outcome: 'pass', text: '' });
  });

  it('2, 3 and 4 open the note box for fail, partial and blocked and post nothing', async () => {
    const { calls } = await mountTab(board());
    const button = await rowButton();
    for (const [key, outcome] of [['2', 'fail'], ['3', 'partial'], ['4', 'blocked']]) {
      fireEvent.keyDown(button, { key });
      expect(noteBox()).toBeTruthy();
      expect(screen.getByRole('button', { name: `Send ${outcome}` })).toBeTruthy();
    }
    expect(resultCalls(calls)).toHaveLength(0);
  });

  it('are typing, not shortcuts, inside the note box', async () => {
    await mountTab(board());
    fireEvent.keyDown(await rowButton(), { key: '2' });
    fireEvent.keyDown(noteBox(), { key: '3' });
    expect(screen.getByRole('button', { name: 'Send fail' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Send partial' })).toBeNull();
  });

  it('do nothing with a modifier held, or on a test that is done', async () => {
    const { calls } = await mountTab({ builds: board().builds, tests: [doneDoc('T-1', 'pass', 'ok'), ...board().tests.slice(1)] });
    fireEvent.keyDown(await rowButton('T-2'), { key: '1', ctrlKey: true });
    fireEvent.keyDown(await rowButton('T-1'), { key: '1' });
    expect(resultCalls(calls)).toHaveLength(0);
  });
});

describe('the key m on a test row', () => {
  const mixed = () => ({ builds: [buildDoc('bundle-1')], tests: [testDoc('T-1'), doneDoc('T-2', 'pass', 'ok')] });

  it('opens the row menu of a test to run and of a done test', async () => {
    await mountTab(mixed());
    fireEvent.keyDown(await rowButton('T-1'), { key: 'm' });
    expect(await screen.findByRole('menuitem', { name: 'Open details' })).toBeTruthy();
    expect(screen.queryByRole('menuitem', { name: 'Run again' })).toBeNull();
    fireEvent.keyDown(screen.getByRole('menuitem', { name: 'Open details' }), { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('menuitem', { name: 'Open details' })).toBeNull());
    fireEvent.keyDown(await rowButton('T-2'), { key: 'M' });
    expect(await screen.findByRole('menuitem', { name: 'Run again' })).toBeTruthy();
  });

  it('is typing inside the note box, and does nothing with a modifier held', async () => {
    await mountTab(board());
    const box = await openBox('Fail');
    fireEvent.keyDown(box, { key: 'm' });
    fireEvent.keyDown(await rowButton('T-2'), { key: 'm', ctrlKey: true });
    expect(screen.queryByRole('menuitem')).toBeNull();
  });

  it('runs the menu item chosen with the keyboard', async () => {
    await mountTab(mixed());
    fireEvent.keyDown(await rowButton('T-1'), { key: 'm' });
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Open details' }));
    expect(await screen.findByRole('dialog', { name: 'Test details' })).toBeTruthy();
  });
});

describe('after a result on a row', () => {
  const focusedName = (): string => document.activeElement?.getAttribute('aria-label') ?? '';

  it('moves the focus to the first button of the next test to run', async () => {
    const { calls } = await mountTab(board());
    fireEvent.click(await screen.findByRole('button', { name: 'Pass T-1' }));
    await firstResult(calls);
    await waitFor(() => expect(focusedName()).toBe('Pass T-2'));
  });

  it('moves on after the keys too, and after a fail sent with its note', async () => {
    const { calls } = await mountTab({ builds: [buildDoc('bundle-1')], tests: [testDoc('T-1'), testDoc('T-2'), testDoc('T-3')] });
    fireEvent.keyDown(await rowButton('T-1'), { key: '1' });
    await firstResult(calls);
    await waitFor(() => expect(focusedName()).toBe('Pass T-2'));
    const box = await openBox('Fail', 'T-2');
    type(box, 'broken');
    send(box);
    await waitFor(() => expect(focusedName()).toBe('Pass T-3'));
  });

  it('keeps the focus where it is on the last test to run', async () => {
    const { calls } = await mountTab(board());
    const last = await screen.findByRole('button', { name: 'Pass T-2' });
    last.focus();
    fireEvent.click(last);
    await firstResult(calls);
    expect(focusedName()).toBe('Pass T-2');
  });

  it('does not move on when the board refuses the result', async () => {
    const { calls } = await mountTab(board(), refuseResults);
    fireEvent.click(await screen.findByRole('button', { name: 'Pass T-1' }));
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect(focusedName()).not.toBe('Pass T-2');
  });
});
