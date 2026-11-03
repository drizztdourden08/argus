/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { inTab } from './kanban-drawer-dom';
import type { Doc } from './kanban-harness';
import { HOUR, ago, cleanKanban, item, mountKanban, openItem, paneOf } from './kanban-harness';

cleanKanban();

const leaf = (id: string, type: string, title: string): Doc => ({
  project: 'alpha', item: id, title, type, lane: 'open', stage: null, assignee: null, edge: { kind: type === 'question' ? 'ask' : type, id, state: 'open', since: null }, cycle: false, children: [],
});

const seats: Doc = {
  project: 'tessera', item: 'I-70', title: 'Seats', type: 'task', lane: 'doing', stage: 'doing', assignee: 'lead:tessera',
  edge: { kind: 'dep-waits', id: 'D-4', state: 'open', since: ago(2 * HOUR) }, cycle: false,
  children: [{
    project: 'brock', item: 'I-105', title: 'Tokens', type: 'task', lane: 'todo', stage: 'todo', assignee: 'lead:brock',
    edge: { kind: 'dep-waits', id: 'D-5', state: 'open', since: null }, cycle: false, children: [],
  }],
};

const local: Doc = {
  project: 'alpha', item: 'I-2', title: 'Wire the lobby', type: 'task', lane: 'doing', stage: 'doing', assignee: 'worker',
  edge: { kind: 'item', id: 'I-2', state: 'doing', since: null }, cycle: false, children: [],
};

const looping: Doc = { ...local, item: 'I-1', title: 'Lobby', cycle: true, edge: { kind: 'item', id: 'I-1', state: 'todo', since: null } };

const trail = (children: Doc[], depth = 3): Doc => ({
  depth,
  open: 3,
  root: { project: 'alpha', item: 'I-1', title: 'Lobby', type: 'bug', lane: 'todo', stage: 'reported', assignee: 'worker', edge: null, cycle: false, children },
});

const causes = [
  { kind: 'dep-waits', id: 'D-4', label: 'Seats', who: 'lead:tessera' },
  { kind: 'item', id: 'I-2', label: 'Wire the lobby' },
  { kind: 'ask', id: 'Q-4', label: 'Which seat?' },
];

const blockedBug = (): Doc =>
  item('I-1', { type: 'bug', stage: 'reported', title: 'Lobby', blockedBy: ['I-2'], blocked: causes });

const withTrail = (body: Doc | null) => {
  const urls: string[] = [];
  vi.stubGlobal('fetch', vi.fn((url: string) => {
    urls.push(url);
    return Promise.resolve(new Response(JSON.stringify(url.endsWith('/trail') ? (body ?? {}) : { id: 'entry', status: 'applied' })));
  }));
  return urls;
};

const linksOf = async (body: Doc | null, extra: Doc[] = []) => {
  await mountKanban({ items: [blockedBug(), item('I-2', { title: 'Wire the lobby' }), ...extra] });
  const urls = withTrail(body);
  const dialog = inTab(await openItem('I-1'), 'Links');
  return { dialog, urls };
};

describe('I-77 P6 the dependency trail in the Links tab', { tags: ['standard'] }, () => {
  it('asks the server for the trail of the open item, once the Links tab is open', async () => {
    const { urls } = await linksOf(trail([seats]));
    await waitFor(() => expect(urls).toContain('/api/p/alpha/items/I-1/trail'));
  });

  it('shows the chain as an indented tree with its depth and open count', async () => {
    const { dialog } = await linksOf(trail([seats, leaf('Q-4', 'question', 'Which seat?')]));
    const tree = await within(dialog).findByRole('list', { name: 'Dependency trail' });
    expect(within(dialog).getByRole('region', { name: 'Dependency trail · 3 deep, 3 open' })).toBeTruthy();
    expect(within(tree).getByRole('button', { name: 'Open tessera/K-70: Seats' })).toBeTruthy();
    expect(within(tree).getByRole('button', { name: 'Open brock/K-105: Tokens' })).toBeTruthy();
    expect(within(tree).getByRole('button', { name: 'Open Q-4: Which seat?' })).toBeTruthy();
    const branch = within(tree).getByRole('list', { name: 'Waits on, K-70' });
    expect(within(branch).getByText('Tokens')).toBeTruthy();
    expect(within(tree).getByRole('list', { name: 'Waits on, B-1' })).toBeTruthy();
  });

  it('says how each node is reached: the kind of link, its id, its state and its age', async () => {
    const { dialog } = await linksOf(trail([seats]));
    const tree = await within(dialog).findByRole('list', { name: 'Dependency trail' });
    expect(within(tree).getByText('request · D-4 · open · 2h ago')).toBeTruthy();
  });

  it('marks a node that is already on the path, and still lets it be opened', async () => {
    const { dialog } = await linksOf(trail([{ ...local, children: [looping] }]));
    const tree = await within(dialog).findByRole('list', { name: 'Dependency trail' });
    expect(within(tree).getByText('cycle')).toBeTruthy();
  });

  it('opens the pane of a node of this board when it is clicked', async () => {
    const { dialog } = await linksOf(trail([local], 2));
    const tree = await within(dialog).findByRole('list', { name: 'Dependency trail' });
    fireEvent.click(within(tree).getByRole('button', { name: 'Open K-2: Wire the lobby' }));
    await waitFor(() => expect(paneOf().open).toBe('K-2'));
  });

  it('opens a node of another board through the link of that board', async () => {
    const { dialog } = await linksOf(trail([seats]));
    const tree = await within(dialog).findByRole('list', { name: 'Dependency trail' });
    fireEvent.click(within(tree).getByRole('button', { name: 'Open tessera/K-70: Seats' }));
    await waitFor(() => expect(paneOf().open).toBe('I-70'));
  });

});

describe('I-77 P6 the start and the absence of the trail', { tags: ['standard'] }, () => {
  it('offers no click on the start of the trail', async () => {
    const { dialog } = await linksOf(trail([local], 2));
    const tree = await within(dialog).findByRole('list', { name: 'Dependency trail' });
    expect(within(tree).queryByRole('button', { name: 'Open B-1: Lobby' })).toBeNull();
    expect(within(tree).getByText('Lobby')).toBeTruthy();
  });

  it('says so when the server sends no trail', async () => {
    const { dialog } = await linksOf(null);
    expect(await within(dialog).findByText('No trail to show yet.')).toBeTruthy();
  });

  it('asks for no trail and says nothing holds an item that is not blocked', async () => {
    await mountKanban({ items: [item('I-1')] });
    const urls = withTrail(trail([local]));
    const dialog = inTab(await openItem('I-1'), 'Links');
    expect(within(dialog).getByText('Nothing holds this item back.')).toBeTruthy();
    expect(urls.filter((url) => url.endsWith('/trail'))).toHaveLength(0);
  });

  it('lists what holds the item back on this board, and what it holds back', async () => {
    const holding = item('I-3', { title: 'Needs the lobby', blockedBy: ['I-1'] });
    const { dialog } = await linksOf(trail([local], 2), [holding]);
    const section = within(within(dialog).getByRole('region', { name: 'Blocked by and blocks' }));
    expect(section.getByText('Wire the lobby')).toBeTruthy();
    expect(section.getByText('Which seat?')).toBeTruthy();
    expect(section.getByText('Needs the lobby')).toBeTruthy();
    expect(section.getByText('blocks')).toBeTruthy();
    await screen.findByRole('list', { name: 'Dependency trail' });
  });
});
