/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { NOW, cleanEntryBoard, entryWork, mountEntryBoard } from '../helpers/entry-board';

cleanEntryBoard();

const work = entryWork({ blocked: [{ kind: 'ask', id: 'Q-1', label: 'Ship it today?' }] });
const question = { id: 'Q-1', kind: 'question', item: 'I-1', thread: 'worker', author: 'worker', status: 'open', text: 'Ship it today?', createdAt: NOW };

const trail = {
  depth: 2,
  open: 2,
  root: {
    project: 'rotp', item: 'I-1', title: 'Item I-1', type: 'task', lane: 'doing', stage: null, assignee: 'worker', edge: null, cycle: false,
    children: [{
      project: 'rotp', item: 'Q-1', title: 'Ship it today?', type: 'question', lane: 'open', stage: null, assignee: 'worker',
      edge: { kind: 'ask', id: 'Q-1', state: 'open', since: null }, cycle: false, children: [],
    }],
  },
};

const mount = () =>
  mountEntryBoard({ at: '/p/rotp/#I-1/links', work, entries: { 'Q-1': question }, body: (url) => (url.endsWith('/trail') ? trail : {}) });

describe('I-77 P6 a leaf of the trail', { tags: ['standard'] }, () => {
  it('opens the entry pane when a question at the end of the trail is clicked', async () => {
    await mount();
    const drawer = await screen.findByRole('dialog', { name: 'Work item' });
    const tree = await within(drawer).findByRole('list', { name: 'Dependency trail' });
    fireEvent.click(within(tree).getByRole('button', { name: 'Open Q-1: Ship it today?' }));
    const pane = await screen.findByRole('dialog', { name: 'Entry detail' });
    expect(within(pane).getByRole('heading', { name: 'Q-1' })).toBeTruthy();
    expect(within(pane).getByText('Ship it today?')).toBeTruthy();
  });
});
