/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { rich } from './kanban-drawer-dom';
import { MIN, ago, cleanKanban, mountKanban, openItem } from './kanban-harness';
import type { Board } from './kanban-harness';

cleanKanban();

const question = {
  id: 'Q-1',
  kind: 'question',
  item: 'I-1',
  thread: 'worker',
  author: 'worker',
  status: 'open',
  text: 'Use `npm test`?',
  context: 'The suite is slow.\n\nIt runs twice.\n\nIt is flaky.',
  createdAt: ago(10 * MIN),
};
const todo = { id: 'H-1', item: 'I-1', thread: 'worker', text: 'Run it by hand', status: 'pending', rank: 1, rev: 2, createdAt: ago(20 * MIN) };

const mountOpen = async (board: Board = {}) => {
  const mounted = await mountKanban({ items: [rich()], ...board });
  const dialog = await openItem('I-1');
  return { ...mounted, dialog };
};

describe('the to-dos of an item', () => {
  it('lists them with the open count, and adds one for the thread of the item', async () => {
    const { posted, dialog } = await mountOpen({ actions: [todo] });
    expect(within(dialog).getByText('To-dos · 1 open')).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Edit to-do H-1: Run it by hand' })).toBeTruthy();
    fireEvent.change(within(dialog).getByRole('textbox', { name: '+ To-do for worker' }), { target: { value: 'Write it down' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add to-do' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'create', target: { c: 'actions' }, set: { text: 'Write it down', item: 'I-1', thread: 'worker' } });
    expect(await screen.findByText('To-do added for worker.')).toBeTruthy();
  });

  it('marks one done with a done entry, and reopens a done one with an edit', async () => {
    const { posted, dialog } = await mountOpen({ actions: [todo, { ...todo, id: 'H-2', text: 'Already', status: 'done' }] });
    fireEvent.click(within(within(dialog).getByRole('region', { name: 'To-dos · 1 open' })).getByRole('button', { name: 'Mark H-1 done' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'done', replyTo: 'H-1', item: 'I-1', thread: 'worker' });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Reopen H-2' }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'edit', target: { c: 'actions', id: 'H-2' }, set: { status: 'pending' }, prev: { status: 'done' }, baseRev: 2 });
  });

  it('edits the text of a to-do in place', async () => {
    const { posted, dialog } = await mountOpen({ actions: [todo] });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit to-do H-1: Run it by hand' }));
    const box = within(dialog).getByRole('textbox', { name: 'Edit to-do H-1' });
    fireEvent.change(box, { target: { value: 'Run it twice' } });
    fireEvent.keyDown(box, { key: 'Enter' });
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'edit', target: { c: 'actions', id: 'H-1' }, set: { text: 'Run it twice' }, prev: { text: 'Run it by hand' }, baseRev: 2 });
  });

  it('refuses a to-do over 200 characters', async () => {
    const { posted, dialog } = await mountOpen();
    fireEvent.change(within(dialog).getByRole('textbox', { name: '+ To-do for worker' }), { target: { value: 'x'.repeat(201) } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add to-do' }));
    expect(within(dialog).getByText('At most 200 characters.')).toBeTruthy();
    expect(posted).toHaveLength(0);
  });
});

describe('the questions and decisions of an item', () => {
  it('shows an open question with its context, and answers it with a followup', async () => {
    const { posted, dialog } = await mountOpen({ entries: [question] });
    const section = within(within(dialog).getByRole('region', { name: 'Questions · 1 open' }));
    expect(section.getByText('npm test').tagName).toBe('CODE');
    expect(section.getByText('The suite is slow.')).toBeTruthy();
    expect(section.getByText('Show the rest')).toBeTruthy();
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Answer worker.' }), { target: { value: 'Yes' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Answer' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'followup', replyTo: 'Q-1', item: 'I-1', thread: 'worker', text: 'Yes' });
  });

  it('shows the owner reply under a question that is answered, with no form', async () => {
    const reply = { id: 'O-1', kind: 'followup', item: 'I-1', thread: 'worker', author: 'human', status: 'new', replyTo: 'Q-1', text: 'Yes', createdAt: ago(MIN) };
    const { dialog } = await mountOpen({ entries: [question, reply] });
    expect(within(dialog).getByText('Questions · 0 open')).toBeTruthy();
    expect(within(dialog).getByRole('group', { name: 'Reply O-1' })).toBeTruthy();
    expect(within(dialog).queryByRole('textbox', { name: 'Answer worker.' })).toBeNull();
  });

  it('lists the decisions, and the pending decision of the owner is editable', async () => {
    const decision = { id: 'D-1', kind: 'decision', item: 'I-1', thread: 'worker', author: 'human', status: 'pending', text: 'Ship it', createdAt: ago(MIN) };
    const { posted, dialog } = await mountOpen({ entries: [decision] });
    expect(within(dialog).getByText('waiting for worker to pick it up')).toBeTruthy();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit decision D-1: Ship it' }));
    const box = within(dialog).getByRole('textbox', { name: 'Edit decision D-1' });
    fireEvent.change(box, { target: { value: 'Ship it Friday' } });
    fireEvent.blur(box);
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'edit', target: { c: 'entries', id: 'D-1' }, set: { text: 'Ship it Friday' }, prev: { text: 'Ship it' } });
  });

  it('says there is none when nothing was asked or decided', async () => {
    const { dialog } = await mountOpen();
    expect(within(dialog).getByText('No question on this item.')).toBeTruthy();
    expect(within(dialog).getByText('No decision recorded yet.')).toBeTruthy();
  });
});

describe('the activity of an item', () => {
  const log = [{ id: 'work', subject: 'work', entries: [{ seq: 1, ts: ago(5 * MIN), thread: 'worker', item: 'I-1', text: 'Started the form' }, { seq: 2, ts: ago(MIN), thread: 'worker', item: 'I-9', text: 'Elsewhere' }] }];

  it('merges the entries, the to-dos and the log lines of this item, newest first', async () => {
    const { dialog } = await mountOpen({ entries: [question], actions: [todo], log });
    expect(within(dialog).getByText('Activity · 3')).toBeTruthy();
    const rows = within(within(dialog).getByRole('list', { name: 'Activity' })).getAllByRole('listitem');
    expect(rows.map((row) => row.textContent)).toEqual([
      expect.stringContaining('Started the form'),
      expect.stringContaining('Use npm test?'),
      expect.stringContaining('Run it by hand'),
    ]);
    expect(within(dialog).queryByText('Elsewhere')).toBeNull();
  });

  it('has the composer box as the markdown editor with its toolbar', async () => {
    const { dialog } = await mountOpen();
    const box = within(dialog).getByRole<HTMLTextAreaElement>('textbox', { name: 'The decision for worker' });
    expect(box.maxLength).toBe(50000);
    expect(within(dialog).getAllByRole('toolbar', { name: 'Formatting' }).length).toBeGreaterThan(0);
  });

  it('posts a decision for the thread from the composer, and a note when Note is picked', async () => {
    const { posted, dialog } = await mountOpen();
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'The decision for worker' }), { target: { value: 'Use the new form' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Post decision' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'decision', text: 'Use the new form', item: 'I-1', thread: 'worker' });
    expect(await screen.findByText('Decision queued for worker.')).toBeTruthy();
    fireEvent.click(within(dialog).getByRole('radio', { name: 'Note' }));
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'The note for worker' }), { target: { value: 'FYI' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Post note' }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'followup', text: 'FYI', item: 'I-1' });
  });

  it('posts nothing for an empty box and says what to write', async () => {
    const { posted, dialog } = await mountOpen();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Post decision' }));
    expect(await screen.findByText('Write the decision.')).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('marks a pending manual step done from the timeline', async () => {
    const { posted, dialog } = await mountOpen({ actions: [todo] });
    const list = within(dialog).getByRole('list', { name: 'Activity' });
    fireEvent.click(within(list).getByRole('button', { name: 'Mark H-1 done' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'done', replyTo: 'H-1' });
  });
});
