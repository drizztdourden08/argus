/* @layer renderer-app @kind test */
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { byId, decision, dismissal, manualStep, message, noContext, notice } from '../helpers/board-docs';
import { openBoard, resetBoard, textbox } from '../helpers/open-board';
import { mockPosts, sentPosts } from '../helpers/post-mock';
import { changeBoard } from '../helpers/seed-board';

afterEach(resetBoard);

describe('S1 information is dismissed', { tags: ['standard'] }, () => {
  it('a notice has Dismiss and no field, and dismissing posts an ack', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(notice) });
    expect(screen.getByText('Notices · 1')).toBeTruthy();
    expect(screen.queryByRole('textbox')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(sentPosts(fetchMock)[0]?.body).toMatchObject({ kind: 'ack', text: '', replyTo: 'N-1', thread: 'coordinator', item: null });
    expect(await screen.findByText('Dismissed.')).toBeTruthy();
  });

  it('a message also keeps Reply', async () => {
    const fetchMock = mockPosts();
    await openBoard({ entries: byId(message) });
    expect(screen.getByRole('button', { name: 'Dismiss' })).toBeTruthy();
    fireEvent.change(textbox('Reply to coordinator.'), { target: { value: 'thanks' } });
    fireEvent.click(screen.getByRole('button', { name: 'Reply' }));
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(sentPosts(fetchMock)[0]?.body).toMatchObject({ kind: 'followup', text: 'thanks', replyTo: 'M-1' });
  });

  it('a dismissed card leaves the feed once the ack is on the stream', async () => {
    mockPosts();
    const view = await openBoard({ entries: byId(notice) });
    expect(screen.getByText('Crash restored')).toBeTruthy();
    act(() => changeBoard(view.sources[0], 'entries', 'O-9', dismissal('N-1')));
    await waitFor(() => expect(screen.queryByText('Crash restored')).toBeNull());
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

describe('S1 manual steps', { tags: ['standard'] }, () => {
  it('a manual step has Done, which posts done for the step and hides it before the board applied it', async () => {
    const fetchMock = mockPosts();
    const view = await openBoard({ actions: byId(manualStep) });
    expect(screen.getByText('manual step')).toBeTruthy();
    expect(screen.getByText('Restart the app')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Done' }));
    await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
    expect(sentPosts(fetchMock)[0]?.body).toMatchObject({ kind: 'done', replyTo: 'H-1', thread: 'coordinator', item: 'I-4', text: '' });
    expect(await screen.findByText('H-1 marked done.')).toBeTruthy();
    act(() => changeBoard(view.sources[0], 'entries', 'O-8', { ...dismissal('H-1', 'O-8'), kind: 'done' }));
    await waitFor(() => expect(screen.queryByText('Restart the app')).toBeNull());
  });
});
