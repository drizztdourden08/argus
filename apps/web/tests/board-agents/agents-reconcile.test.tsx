/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { agent, ago, DAY, MIN, mockApi, mockFetch, renderAgents, rowOf } from './agents-harness';

afterEach(() => {
  localStorage.clear();
  cleanup();
  vi.unstubAllGlobals();
});

const finding = (extra: Record<string, unknown> = {}) => ({
  project: 'alpha',
  id: 'A-8',
  name: 'wake',
  thread: 'lead',
  class: 'idle',
  detail: 'turn ended 0 min ago',
  ...extra,
});

describe('Agents tab: reconciler marks and findings', () => {
  it('marks a stale record on its row and the idle and orphan ones in their drawers', async () => {
    mockApi();
    const { container, unmount } = await renderAgents({ agents: { 'A-3': agent({ name: 'old', updatedAt: ago(50 * MIN) }) } });
    expect(within(rowOf(container, 'A-3')).getByText('stale')).toBeTruthy();
    unmount();
    for (const [id, extra, mark] of [['A-1', { idleSince: ago(6 * MIN) }, 'idle'], ['A-2', { orphanSince: ago(20 * MIN) }, 'orphan']] as const) {
      mockApi();
      const mounted = await renderAgents({ agents: { [id]: agent({ name: 'marked', ...extra }) } }, () => undefined, `/p/alpha#${id}`);
      expect(within(await screen.findByRole('dialog', { name: 'Agent detail' })).getByText(mark)).toBeTruthy();
      mounted.unmount();
    }
  });

  it('marks a duplicate, a lost record and the mismatches in the drawer of a finished record', async () => {
    const cases = [
      ['A-1', { status: 'stopped', duplicateOf: 'A-9' }, ['duplicate of A-9']],
      ['A-2', { status: 'stopped', lost: true }, ['lost']],
      ['A-3', { status: 'done', idMismatch: true, modelMismatch: true, modelUsed: 'opus-5-5' }, ['id mismatch', 'model mismatch']],
    ] as const;
    for (const [id, extra, marks] of cases) {
      mockApi();
      const mounted = await renderAgents({ agents: { [id]: agent(extra) } }, () => undefined, `/p/alpha#${id}`);
      const dialog = within(await screen.findByRole('dialog', { name: 'Agent detail' }));
      for (const mark of marks) expect(dialog.getByText(mark)).toBeTruthy();
      mounted.unmount();
    }
  });

  it('asks the board for the reconciler report of this project and lists the findings', async () => {
    mockApi({ ran: true, at: ago(MIN), findings: [finding(), finding({ id: 'A-9', name: 'gone', class: 'orphan', detail: 'no task' })] });
    await renderAgents({ agents: { 'A-8': agent({ name: 'wake', idleSince: ago(MIN) }) } });
    expect(await screen.findByText(/The reconciler flagged 2 records/)).toBeTruthy();
    expect(screen.getByText('wake (A-8)')).toBeTruthy();
    expect(screen.getByText('gone (A-9)')).toBeTruthy();
    expect(screen.getByText('turn ended 0 min ago')).toBeTruthy();
    const fetchMock = vi.mocked(fetch);
    expect(fetchMock.mock.calls[0]?.[0]).toBe('/api/agents/reconcile?project=alpha');
  });

  it('shows no panel when the report is empty or the route is missing', async () => {
    mockFetch(() => new Response('{}', { status: 404 }));
    await renderAgents({ agents: { 'A-1': agent() } });
    await waitFor(() => expect(vi.mocked(fetch)).toHaveBeenCalled());
    expect(screen.queryByText(/The reconciler flagged/)).toBeNull();
  });
});

describe('Agents tab: the lead and the chats', () => {
  const threads = {
    lead: { name: 'lead', role: 'coordinator', status: 'active', activity: 'working', note: 'dispatching', session: 'Argus', lastSeen: ago(MIN), link: 'chat://x/lead' },
    chatty: { name: 'chatty', role: 'thread', status: 'active', activity: 'working', note: 'talking', lastSeen: ago(MIN), link: 'chat://x/chat' },
    waiting: { name: 'waiting', role: 'thread', status: 'blocked', activity: 'waiting', note: null },
    finished: { name: 'finished', role: 'thread', status: 'done', activity: 'idle' },
    old: { name: 'old', role: 'thread', status: 'active', archivedAt: ago(DAY) },
    merged: { name: 'merged', role: 'thread', status: 'active', mergedInto: 'chatty' },
  };

  it('puts the lead, its note and its chat link in the header', async () => {
    mockApi();
    await renderAgents({ agents: { 'A-1': agent() }, threads });
    expect(screen.getByText('dispatching')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Open the chat of the lead' })).toBeTruthy();
  });

  it('lists the owner chats in a compact section, blocked first, and hides finished ones', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: {}, threads });
    const ids = [...container.querySelectorAll('[data-chat-id]')].map((row) => row.getAttribute('data-chat-id'));
    expect(ids).toEqual(['waiting', 'chatty']);
  });

  it('puts Purge in the menu of a chat, which asks to purge that thread', async () => {
    mockApi();
    vi.stubGlobal('CSS', { supports: () => false });
    const onPurge = vi.fn();
    await renderAgents({ agents: {}, threads }, onPurge);
    fireEvent.click(screen.getByRole('button', { name: 'More actions for chatty' }));
    expect(await screen.findByRole('menuitem', { name: 'Copy id' })).toBeTruthy();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Purge...' }));
    expect(onPurge).toHaveBeenCalledWith('chatty');
  });
});
