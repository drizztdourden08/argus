/* @layer renderer-app @kind test */
import { cleanup, fireEvent, within } from '@testing-library/react';
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
  it('clamps the last message to plain text and opens the whole message rendered', async () => {
    const message = 'Fixed the **form** and `ran` the tests.\n\n- 12 passed\n- 0 failed';
    const { container } = await opened({ 'A-1': agent({ name: 'coder', live: live({ lastMessage: message, activity: 'thinking', activitySince: ago(1000) }) }) });
    const row = rowOf(container, 'A-1');
    expect(row.textContent).toContain('Fixed the form and ran the tests. 12 passed 0 failed');
    expect(row.textContent).not.toMatch(/\*\*|`/);
    fireEvent.click(within(row).getByTitle('Read the whole message'));
    const whole = container.querySelector<HTMLElement>('[data-agent-message="A-1"]');
    expect(whole?.querySelector('strong')?.textContent).toBe('form');
    expect(within(whole as HTMLElement).getByText('12 passed')).toBeTruthy();
    fireEvent.click(within(whole as HTMLElement).getByRole('button', { name: 'Close' }));
    expect(container.querySelector('[data-agent-message]')).toBeNull();
  });

  it('draws a message with markup as text', async () => {
    const { container } = await opened({ 'A-1': agent({ live: live({ lastMessage: '<img src=x onerror=alert(1)> done' }) }) });
    expect(container.querySelector('img')).toBeNull();
    expect(rowOf(container, 'A-1').textContent).toContain('<img src=x onerror=alert(1)> done');
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
