/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { NOW, cleanEntryBoard, entryWork, mountEntryBoard } from '../helpers/entry-board';

cleanEntryBoard();

const work = entryWork();
const question = {
  id: 'Q-1',
  kind: 'question',
  item: 'I-1',
  thread: 'worker',
  author: 'worker',
  status: 'open',
  text: 'Ship it today?',
  context: 'The suite is green.',
  options: ['Ship now', 'Wait'],
  createdAt: NOW,
};
const note = { id: 'F-1', kind: 'followup', replyTo: 'Q-1', item: 'I-1', thread: 'worker', author: 'worker', text: 'An earlier note', createdAt: NOW };

const mount = (at: string) => mountEntryBoard({ at, work, entries: { 'Q-1': question, 'F-1': note } });

const paneOf = async (): Promise<HTMLElement> => screen.findByRole('dialog', { name: 'Entry detail' });

const expectDetail = (pane: HTMLElement) => {
  const inside = within(pane);
  expect(inside.getByRole('heading', { name: 'Q-1' })).toBeTruthy();
  expect(inside.getByText('Ship it today?')).toBeTruthy();
  expect(inside.getByText('The suite is green.')).toBeTruthy();
  expect(inside.getByRole('button', { name: /^Ship now/ })).toBeTruthy();
  expect(inside.getByText('open')).toBeTruthy();
  expect(within(inside.getByRole('group', { name: 'Reply F-1' })).getByText('An earlier note')).toBeTruthy();
  expect(inside.getByRole('button', { name: 'Open I-1' })).toBeTruthy();
};

describe('I-68 the entry pane', { tags: ['standard'] }, () => {
  it('opens from the question tag of a card, shows the entry in full, and closes with Escape', async () => {
    await mount('/p/rotp/#kanban');
    fireEvent.click(await screen.findByTitle('Q-1 is an open question, waiting for an answer'));
    const pane = await paneOf();
    expectDetail(pane);
    fireEvent.keyDown(pane, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Entry detail' })).toBeNull());
  });

  it('opens from the id of a card of the owner feed', async () => {
    await mount('/p/rotp/#activity');
    fireEvent.click(await screen.findByRole('button', { name: 'Open Q-1' }));
    expectDetail(await paneOf());
  });

  it('opens from the id of an entry in the drawer of the item', async () => {
    await mount('/p/rotp/#I-1');
    const drawer = await screen.findByRole('dialog', { name: 'Work item' });
    fireEvent.click(within(drawer).getByRole('tab', { name: /^Discussion/ }));
    const links = await within(drawer).findAllByRole('button', { name: 'Open Q-1' });
    fireEvent.click(links[0] as HTMLElement);
    expectDetail(await paneOf());
  });
});
