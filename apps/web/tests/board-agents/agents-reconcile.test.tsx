/* @layer renderer-app @kind test */
import { cleanup, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { agent, ago, MIN, mockApi, renderAgents } from './agents-harness';

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
