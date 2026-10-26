/* @layer renderer-app @kind test */
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { NOW, byId, decision } from '../helpers/board-docs';
import { openBoard, resetBoard, textbox } from '../helpers/open-board';
import { mockPosts, onlyPost, sentPosts } from '../helpers/post-mock';

afterEach(resetBoard);

const item = (id: string, over: object = {}) => ({
  id,
  title: `Item ${id}`,
  status: 'doing',
  thread: 'worker',
  description: 'What **this** is about.',
  rank: 1,
  ac: [
    { n: 1, text: 'first criterion', done: true },
    { n: 2, text: 'second criterion', done: false },
  ],
  updatedAt: NOW,
  ...over,
});

describe('the work items', () => {
  it('lists the open items with their status, thread and criteria met, and hides finished ones', async () => {
    mockPosts();
    await openBoard({
      items: byId(item('I-1'), item('I-2', { status: 'done' }), item('I-3', { status: 'dropped' }), item('I-4', { archivedAt: NOW })),
    });
    expect(screen.getByText('1 open')).toBeTruthy();
    expect(screen.getByText('Item I-1')).toBeTruthy();
    expect(screen.getByText('Doing')).toBeTruthy();
    expect(screen.getByText('worker')).toBeTruthy();
    expect(screen.getByText('1/2 criteria')).toBeTruthy();
    expect(screen.queryByText('Item I-2')).toBeNull();
    expect(screen.queryByText('Item I-3')).toBeNull();
    expect(screen.queryByText('Item I-4')).toBeNull();
  });

  it('says when nothing is open', async () => {
    mockPosts();
    await openBoard({ items: byId(item('I-2', { status: 'done' })) });
    expect(screen.getByText('No open work item.')).toBeTruthy();
    expect(screen.getByText('0 open')).toBeTruthy();
  });

  it('shows the description as markdown and every criterion, met ones ticked', async () => {
    mockPosts();
    await openBoard({ items: byId(item('I-1')) });
    expect(screen.getByText('this').tagName).toBe('STRONG');
    expect(screen.getByText('Acceptance criteria · 1/2')).toBeTruthy();
    const first = screen.getByRole<HTMLInputElement>('checkbox', { name: 'first criterion' });
    const second = screen.getByRole<HTMLInputElement>('checkbox', { name: 'second criterion' });
    expect(first.checked).toBe(true);
    expect(second.checked).toBe(false);
    expect(first.disabled).toBe(true);
  });

  it('marks an item that waits on the owner', async () => {
    mockPosts();
    await openBoard({ items: byId(item('I-4')), entries: byId(decision) });
    expect(screen.getByText('needs you')).toBeTruthy();
  });

  it('follows the stream: a criterion that gets met redraws the count', async () => {
    mockPosts();
    const view = await openBoard({ items: byId(item('I-1')) });
    const met = item('I-1', { ac: [{ n: 1, text: 'first criterion', done: true }, { n: 2, text: 'second criterion', done: true }] });
    act(() => view.sources[0]?.emit('changes', { scope: 'rotp', seq: 11, changes: [{ coll: 'items', id: 'I-1', op: 'set', data: met }] }));
    expect(screen.getByText('2/2 criteria')).toBeTruthy();
  });
});

describe('a note on an item', () => {
  it('a decision is the default and goes to the thread of the item', async () => {
    const fetchMock = mockPosts();
    await openBoard({ items: byId(item('I-1')) });
    fireEvent.change(textbox('The decision for worker.'), { target: { value: '  Use the small one  ' } });
    fireEvent.click(screen.getByRole('button', { name: 'Post decision' }));
    const post = await onlyPost(fetchMock);
    expect(post?.body).toMatchObject({ kind: 'decision', text: 'Use the small one', item: 'I-1', thread: 'worker', author: 'human' });
    expect(await screen.findByText('Decision queued for worker.')).toBeTruthy();
  });

  it('a note is a followup with more context for the thread, and the box empties', async () => {
    const fetchMock = mockPosts();
    await openBoard({ items: byId(item('I-1')) });
    fireEvent.click(screen.getByRole('radio', { name: 'Note' }));
    const field = textbox('A note for worker.');
    fireEvent.change(field, { target: { value: 'please add a test' } });
    fireEvent.click(screen.getByRole('button', { name: 'Post note' }));
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(sentPosts(fetchMock)[0]?.body).toMatchObject({ kind: 'followup', text: 'please add a test', item: 'I-1', thread: 'worker' });
    expect(await screen.findByText('Note sent to worker.')).toBeTruthy();
    await waitFor(() => expect(field.value).toBe(''));
  });

  it('an empty note posts nothing and says what to write', async () => {
    const fetchMock = mockPosts();
    await openBoard({ items: byId(item('I-1')) });
    fireEvent.click(screen.getByRole('button', { name: 'Post decision' }));
    expect(await screen.findByText('Write the decision.')).toBeTruthy();
    expect(sentPosts(fetchMock)).toHaveLength(0);
  });

  it('Ctrl+Enter posts, a plain Enter does not', async () => {
    const fetchMock = mockPosts();
    await openBoard({ items: byId(item('I-1')) });
    const field = textbox('The decision for worker.');
    fireEvent.change(field, { target: { value: 'go' } });
    fireEvent.keyDown(field, { key: 'Enter' });
    expect(sentPosts(fetchMock)).toHaveLength(0);
    fireEvent.keyDown(field, { key: 'Enter', metaKey: true });
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
  });

  it('a refusal of the board keeps the text in the box', async () => {
    mockPosts('refused', 'item is archived');
    await openBoard({ items: byId(item('I-1')) });
    const field = textbox('The decision for worker.');
    fireEvent.change(field, { target: { value: 'keep me' } });
    fireEvent.click(screen.getByRole('button', { name: 'Post decision' }));
    expect(await screen.findByText('Refused: item is archived.')).toBeTruthy();
    expect(field.value).toBe('keep me');
  });
});
