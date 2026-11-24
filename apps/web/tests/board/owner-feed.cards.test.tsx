/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { byId, decision, freeText } from '../helpers/board-docs';
import { openBoard, resetBoard, textbox } from '../helpers/open-board';
import { mockPosts, onlyPost, sentPosts } from '../helpers/post-mock';

afterEach(resetBoard);

describe('S1 a decision card', { tags: ['standard'] }, () => {
  it('clicking an option decides it and says so', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(decision) });
    fireEvent.click(screen.getByRole('button', { name: 'Yes, private recommended' }));
    const post = await onlyPost(fetchMock);
    expect(post?.body).toMatchObject({
      kind: 'decision',
      text: 'Yes, private',
      replyTo: 'Q-1',
      item: 'I-4',
      thread: 'lead',
      author: 'human',
      status: 'new',
    });
    expect(post?.body.id).toMatch(/^O-[a-z0-9]+-[a-z0-9]{4}$/);
    expect(await screen.findByText('Decided: Yes, private')).toBeTruthy();
  });

  it('a written own answer is sent as a decision, trimmed, and the box empties', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(decision) });
    fireEvent.click(screen.getByRole('button', { name: 'Custom answer' }));
    const field = textbox('Or write your own answer');
    fireEvent.change(field, { target: { value: '  Yes, but public  ' } });
    fireEvent.click(screen.getByRole('button', { name: 'Decide' }));
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(sentPosts(fetchMock)[0]?.body).toMatchObject({ kind: 'decision', text: 'Yes, but public', replyTo: 'Q-1' });
    await waitFor(() => expect(field.value).toBe(''));
  });

  it('a second click while the post is on its way posts once', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(decision) });
    const option = screen.getByRole('button', { name: 'Yes, private recommended' });
    fireEvent.click(option);
    fireEvent.click(option);
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
  });
});

describe('S1 a question without options', { tags: ['standard'] }, () => {
  it('the answer is a followup that points at the question', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(freeText) });
    fireEvent.change(textbox('Answer lead.'), { target: { value: 'Call it argus' } });
    fireEvent.click(screen.getByRole('button', { name: 'Answer' }));
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(sentPosts(fetchMock)[0]?.body).toMatchObject({ kind: 'followup', text: 'Call it argus', replyTo: 'Q-2', thread: 'lead' });
    expect(await screen.findByText('Reply sent to lead.')).toBeTruthy();
  });

  it('Ctrl+Enter sends the box, a plain Enter does not', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(freeText) });
    const field = textbox('Answer lead.');
    fireEvent.change(field, { target: { value: 'line one' } });
    fireEvent.keyDown(field, { key: 'Enter' });
    expect(sentPosts(fetchMock)).toHaveLength(0);
    fireEvent.keyDown(field, { key: 'Enter', ctrlKey: true });
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(sentPosts(fetchMock)[0]?.body.text).toBe('line one');
  });
});

describe('K3 a question without options', { tags: ['standard'] }, () => {
  it('a long answer is posted whole: 20000 characters, no counter, no refusal', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(freeText) });
    fireEvent.change(textbox('Answer lead.'), { target: { value: 'x'.repeat(20000) } });
    expect(screen.queryByRole('alert')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Answer' }));
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(String(sentPosts(fetchMock)[0]?.body.text)).toHaveLength(20000);
  });
});

describe('K4 a question without options', { tags: ['standard'] }, () => {
  it('an empty answer posts nothing and asks for the text', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(freeText) });
    fireEvent.click(screen.getByRole('button', { name: 'Answer' }));
    expect(await screen.findByText('Write the reply.')).toBeTruthy();
    expect(sentPosts(fetchMock)).toHaveLength(0);
  });

  it('over 50000 characters the box says so in a plain sentence and nothing is posted or cut', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(freeText) });
    fireEvent.change(textbox('Answer lead.'), { target: { value: 'x'.repeat(50001) } });
    const sentence = 'This text is 50001 characters and one post takes 50000 at most: shorten it or post it in two parts.';
    expect(screen.getAllByText(sentence)).toHaveLength(1);
    fireEvent.click(screen.getByRole('button', { name: 'Answer' }));
    await waitFor(() => expect(screen.getAllByText(sentence).length).toBeGreaterThan(1));
    expect(sentPosts(fetchMock)).toHaveLength(0);
  });
});
