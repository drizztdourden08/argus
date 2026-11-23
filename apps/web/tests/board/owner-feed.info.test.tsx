/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { byId, decision, message, noContext } from '../helpers/board-docs';
import { openBoard, resetBoard, textbox } from '../helpers/open-board';
import { mockPosts, sentPosts } from '../helpers/post-mock';

afterEach(resetBoard);

describe('S1 a message is dismissed or answered', { tags: ['standard'] }, () => {
  it('a message also keeps Reply', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(message) });
    expect(screen.getByRole('button', { name: 'Dismiss' })).toBeTruthy();
    fireEvent.click(screen.getByText('Write a reply'));
    fireEvent.change(textbox('Reply to coordinator.'), { target: { value: 'thanks' } });
    fireEvent.click(screen.getByRole('button', { name: 'Reply' }));
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(sentPosts(fetchMock)[0]?.body).toMatchObject({ kind: 'followup', text: 'thanks', replyTo: 'M-1' });
  });
});

describe('S1 a question with no context', { tags: ['standard'] }, () => {
  it('says so, never hides, and Ask for context posts a followup to the thread', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(noContext) });
    expect(screen.getByText('Is this clear?')).toBeTruthy();
    expect(screen.getByText('No context was given: ask the thread what this is about')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Ask for context' }));
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(sentPosts(fetchMock)[0]?.body).toMatchObject({
      kind: 'followup',
      text: 'Please add context: what is this about and why does it need me? (Q-3)',
      thread: 'coordinator',
    });
    expect(await screen.findByText('Asked coordinator for context.')).toBeTruthy();
  });
});

describe('S3 the verdict of the board', { tags: ['standard'] }, () => {
  it('a refusal is shown with its reason and the card stays', async () => {
    mockPosts('refused', 'not allowed');
    await openBoard({ entries: byId(decision) });
    fireEvent.click(screen.getByRole('button', { name: 'Yes, private recommended' }));
    expect(await screen.findByText('Refused: not allowed.')).toBeTruthy();
    expect(screen.getByText('Create the private repo?')).toBeTruthy();
  });

  it('a conflict says nothing of yours was applied', async () => {
    mockPosts('conflict');
    await openBoard({ entries: byId(decision) });
    fireEvent.click(screen.getByRole('button', { name: 'Yes, private recommended' }));
    expect(await screen.findByText('Conflict: a thread changed it after you looked. Nothing of yours was applied.')).toBeTruthy();
  });

  it('an unreachable dashboard is told in one sentence', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.reject(new TypeError('offline'))));
    await openBoard({ entries: byId(decision) });
    fireEvent.click(screen.getByRole('button', { name: 'Yes, private recommended' }));
    expect(await screen.findByText('Dashboard unreachable, retrying.')).toBeTruthy();
  });
});
