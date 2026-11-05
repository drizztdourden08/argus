/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { conflictEntry, itemDoc, testDoc } from './fixtures';
import { mountTab } from './mount-tab';
import type { Call } from './mount-tab';

cleanAfterEach();

const REGION = 'Changes the board did not take';

const board = (entries: object[]) => ({
  items: [itemDoc('I-1'), itemDoc('I-2')],
  tests: [testDoc('T-1', { rev: 5 }), testDoc('T-2', { rev: 5 })],
  entries: entries as Record<string, unknown>[],
});

const posted = (calls: Call[]): Call[] => calls.filter((call) => call.url.endsWith('/entries'));

const rowOf = (container: HTMLElement, id: string): HTMLElement => container.querySelector<HTMLElement>(`[data-test="${id}"]`) as HTMLElement;

describe('S1 a change of the owner the board did not take', { tags: ['standard'] }, () => {
  it('says who changed the test after the owner looked, and marks the row', async () => {
    const { container } = await mountTab(board([conflictEntry('O-1', 'T-1')]));
    const region = await screen.findByRole('region', { name: REGION });
    expect(within(region).getByText('Conflict on T-1: lead changed it after you looked. Nothing of yours was applied.')).toBeTruthy();
    expect(within(region).getByText('item')).toBeTruthy();
    expect(within(rowOf(container, 'T-1')).getByText('conflict')).toBeTruthy();
    expect(within(rowOf(container, 'T-2')).queryByText('conflict')).toBeNull();
  });

  it('keeps theirs by acknowledging the entry', async () => {
    const { calls } = await mountTab(board([conflictEntry('O-1', 'T-1')]));
    fireEvent.click(await screen.findByRole('button', { name: 'Keep theirs' }));
    await waitFor(() => expect(posted(calls)).toHaveLength(1));
    expect(posted(calls)[0]?.body).toMatchObject({ kind: 'ack', author: 'human', replyTo: 'O-1' });
  });

  it('uses mine by sending the op again with force against what is there now, then acknowledges it', async () => {
    const { calls } = await mountTab(board([conflictEntry('O-1', 'T-1')]));
    fireEvent.click(await screen.findByRole('button', { name: 'Use mine' }));
    await waitFor(() => expect(posted(calls)).toHaveLength(2));
    expect(posted(calls)[0]?.body).toMatchObject({
      kind: 'move',
      target: { c: 'tests', id: 'T-1' },
      to: { item: 'I-2' },
      prev: { item: 'I-3' },
      baseRev: 5,
      force: true,
      retryOf: 'O-1',
    });
    expect(posted(calls)[1]?.body).toMatchObject({ kind: 'ack', replyTo: 'O-1' });
  });

  it('says a refusal with its reason and dismisses it', async () => {
    const refused = conflictEntry('O-2', 'T-2', { kind: 'edit', set: { title: 'x' }, status: 'refused', outcome: 'not yours' });
    const { calls } = await mountTab(board([refused]));
    const region = await screen.findByRole('region', { name: REGION });
    expect(within(region).getByText('Refused edit on T-2: not yours')).toBeTruthy();
    fireEvent.click(within(region).getByRole('button', { name: 'Dismiss' }));
    await waitFor(() => expect(posted(calls)).toHaveLength(1));
    expect(posted(calls)[0]?.body).toMatchObject({ kind: 'ack', replyTo: 'O-2' });
  });

  it('leaves out an acknowledged entry, a change of an item and one still waiting', async () => {
    const entries = [
      conflictEntry('O-1', 'T-1'),
      { id: 'O-9', kind: 'ack', author: 'human', replyTo: 'O-1', text: '' },
      conflictEntry('O-3', 'I-1', { target: { c: 'items', id: 'I-1' } }),
      conflictEntry('O-4', 'T-2', { status: 'new' }),
    ];
    await mountTab(board(entries));
    await screen.findByText('Test number 1');
    expect(screen.queryByRole('region', { name: REGION })).toBeNull();
  });
});

describe('S1 the conflicted tests in the list', { tags: ['standard'] }, () => {
  it('narrows the list to the conflicted tests with the conflict switch', async () => {
    await mountTab(board([conflictEntry('O-1', 'T-1')]));
    await screen.findByText('Test number 2');
    fireEvent.click(screen.getByRole('checkbox', { name: 'conflict' }));
    expect(screen.queryByText('Test number 2')).toBeNull();
    expect(screen.getByText('Test number 1')).toBeTruthy();
  });
});
