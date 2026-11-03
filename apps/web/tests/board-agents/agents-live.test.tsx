/* @layer renderer-app @kind test */
import { cleanup, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { agent, ago, live, MIN, mockApi, renderAgents, rowOf } from './agents-harness';

afterEach(() => {
  localStorage.clear();
  cleanup();
  vi.unstubAllGlobals();
});

const opened = async (agents: Record<string, object>) => {
  mockApi();
  return renderAgents({ agents });
};

describe('S1 Agents tab: the live view', { tags: ['standard'] }, () => {
  it('shows what a working agent does now on its row', async () => {
    const { container } = await opened({ 'A-1': agent({ live: live({ activity: 'tool', activitySince: ago(2000), lastTool: { name: 'Read', summary: 'a.txt' } }) }) });
    expect(rowOf(container, 'A-1').textContent).toContain('Read a.txt');
  });

  it('marks a stalled working agent and leaves a busy one alone', async () => {
    const { container } = await opened({
      'A-1': agent({ name: 'stuck', live: live({ stalled: true, activity: 'tool', activitySince: ago(5 * MIN), lastActivityAt: ago(4 * MIN), lastTool: { name: 'Bash', summary: 'sleep 999' } }) }),
      'A-2': agent({ name: 'busy', live: live({ activity: 'tool', activitySince: ago(2000), lastTool: { name: 'Read', summary: 'a.txt' } }) }),
      'A-3': agent({ name: 'plain' }),
    });
    expect(within(rowOf(container, 'A-1')).getAllByText(/^stalled 4m$/).length).toBeGreaterThan(0);
    expect(within(rowOf(container, 'A-2')).queryByText(/^stalled/)).toBeNull();
    expect(within(rowOf(container, 'A-3')).queryByText(/^stalled/)).toBeNull();
  });
});
