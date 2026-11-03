/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { agent, ago, DAY, MIN, mockApi, renderAgents } from './agents-harness';

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

describe('S1 Agents tab: reconciler marks and findings', { tags: ['standard'] }, () => {
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
});

describe('S1 Agents tab: the chats', { tags: ['standard'] }, () => {
  const threads = {
    lead: { name: 'lead', role: 'coordinator', status: 'active', activity: 'working', note: 'dispatching', session: 'Argus', lastSeen: ago(MIN), link: 'chat://x/lead' },
    chatty: { name: 'chatty', role: 'thread', status: 'active', activity: 'working', note: 'talking', lastSeen: ago(MIN), link: 'chat://x/chat' },
    waiting: { name: 'waiting', role: 'thread', status: 'blocked', activity: 'waiting', note: null },
    finished: { name: 'finished', role: 'thread', status: 'done', activity: 'idle' },
    old: { name: 'old', role: 'thread', status: 'active', archivedAt: ago(DAY) },
    merged: { name: 'merged', role: 'thread', status: 'active', mergedInto: 'chatty' },
  };

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
