/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { NOW, byId, manualStep } from '../helpers/board-docs';
import { openBoard, resetBoard } from '../helpers/open-board';
import { mockPosts } from '../helpers/post-mock';

afterEach(resetBoard);

const sent = (id: string, thread: string, text: string, over: object = {}) => ({
  id,
  kind: 'followup',
  author: 'human',
  thread,
  status: 'new',
  text,
  createdAt: NOW,
  ...over,
});

const leadStep = { ...manualStep, id: 'H-2', text: 'Rebuild the cache', assignee: 'lead:rotp' };

describe('I-85 the activity keeps what the owner sent to the leads', { tags: ['critical'] }, () => {
  it('counts the unapplied followups with the to-dos and folds them per lead', async () => {
    mockPosts();
    await openBoard({ entries: byId(sent('F-1', 'rotp', 'Check the pane'), sent('F-2', 'rotp', 'And the menu'), sent('F-3', 'brock', 'Ship it')), actions: byId(leadStep) });
    expect(screen.getByText('For the leads · 4')).toBeTruthy();
    expect(screen.getByText('rotp · 2 not read yet')).toBeTruthy();
    expect(screen.getByText('brock · 1 not read yet')).toBeTruthy();
  });

  it('opens a lead fold to show the followups sent to it', async () => {
    mockPosts();
    await openBoard({ entries: byId(sent('F-1', 'rotp', 'Check the pane'), sent('F-2', 'rotp', 'And the menu')) });
    fireEvent.click(screen.getByText('rotp · 2 not read yet'));
    const group = screen.getByRole('group', { name: 'Sent to rotp' });
    expect(within(group).getByText('Check the pane')).toBeTruthy();
    expect(within(group).getByText('And the menu')).toBeTruthy();
  });

  it('leaves a followup out once a lead applied it', async () => {
    mockPosts();
    await openBoard({ entries: byId(sent('F-1', 'rotp', 'Check the pane', { appliedAt: NOW })) });
    expect(screen.queryByText(/For the leads/)).toBeNull();
  });
});
