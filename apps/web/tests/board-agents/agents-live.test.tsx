/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen, within } from '@testing-library/react';
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

describe('Agents tab: the live view', () => {
  it('shows the Now line: the tool with the file name, thinking with its age, waiting without one', async () => {
    const { container } = await opened({
      'A-1': agent({ name: 'editor', live: live({ activity: 'tool', activitySince: ago(12_000), lastTool: { name: 'Edit', summary: 'X:/repo/dashboard/pages/board.html' } }) }),
      'A-2': agent({ name: 'thinker', live: live({ activity: 'thinking', activitySince: ago(4000) }) }),
      'A-3': agent({ name: 'waiter', live: live({ activity: 'waiting', activitySince: ago(4000) }) }),
      'A-4': agent({ name: 'runner', live: live({ activity: 'tool', activitySince: ago(30_000), lastTool: { name: 'Bash', summary: 'npm test' } }) }),
    });
    expect(rowOf(container, 'A-1').textContent).toContain('Edit board.html');
    expect(rowOf(container, 'A-1').textContent).not.toContain('pages');
    expect(rowOf(container, 'A-2').textContent).toContain('thinking');
    expect(rowOf(container, 'A-3').textContent).toContain('waiting');
    expect(rowOf(container, 'A-3').textContent).not.toMatch(/waiting\s*\d/);
    expect(rowOf(container, 'A-4').textContent).toContain('Bash npm test');
  });

  it('clamps the last message to plain text and opens the whole message rendered, with a copy button', async () => {
    const message = 'Fixed the **form** and `ran` the tests.\n\n- 12 passed\n- 0 failed';
    const { container } = await opened({ 'A-1': agent({ name: 'coder', live: live({ lastMessage: message, activity: 'thinking', activitySince: ago(1000) }) }) });
    const row = rowOf(container, 'A-1');
    expect(row.textContent).toContain('Fixed the form and ran the tests. 12 passed 0 failed');
    expect(row.textContent).not.toMatch(/\*\*|`/);
    fireEvent.click(within(row).getByTitle('Read the whole message'));
    const whole = container.querySelector<HTMLElement>('[data-agent-message="A-1"]');
    expect(whole?.querySelector('strong')?.textContent).toBe('form');
    expect(within(whole as HTMLElement).getByText('12 passed')).toBeTruthy();
    expect(within(whole as HTMLElement).getByRole('button', { name: 'Copy the message' })).toBeTruthy();
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

describe('Agents tab: the columns', () => {
  it('shows the elapsed time with turns and tool calls', async () => {
    const { container } = await opened({
      'A-1': agent({ live: live({ activity: 'thinking', activitySince: ago(1000), turns: 7, toolCalls: 12, modelUsed: 'sonnet-5-5', tokens: { input: 12_340, output: 2100, cacheRead: 98_000 } }) }),
    });
    const text = rowOf(container, 'A-1').textContent;
    expect(text).toContain('10m');
    expect(text).toContain('7 \u00b7 12');
  });

  it('shows the model family and keeps the dash for an agent without a model', async () => {
    const { container } = await opened({
      'A-1': agent({ model: 'opus' }),
      'A-2': agent({ model: 'fable-9' }),
      'A-3': agent({ model: null }),
    });
    expect(within(rowOf(container, 'A-1')).getByText('opus')).toBeTruthy();
    expect(within(rowOf(container, 'A-2')).getByText('fable-9')).toBeTruthy();
    expect(within(rowOf(container, 'A-3')).getAllByText('-').length).toBeGreaterThan(0);
  });

  it('shows the final message of a finished agent, and the recorded result without a transcript message', async () => {
    mockApi();
    const { container } = await renderAgents({
      agents: {
        'A-1': agent({ name: 'done-one', status: 'done', finishedAt: ago(MIN), result: 'short result', live: live({ lastMessage: 'All green, 41 tests.', activity: 'waiting' }) }),
        'A-2': agent({ name: 'done-two', status: 'done', finishedAt: ago(2 * MIN), result: 'recorded result' }),
      },
    });
    fireEvent.click(screen.getByRole('button', { name: /Show finished/ }));
    expect(rowOf(container, 'A-1').textContent).toContain('All green, 41 tests.');
    expect(rowOf(container, 'A-1').textContent).not.toContain('stalled');
    expect(rowOf(container, 'A-2').textContent).toContain('recorded result');
  });

  it('shows a note about a missing transcript in the Now line', async () => {
    const { container } = await opened({ 'A-1': agent({ live: live({ liveNote: 'no transcript found', lastActivityAt: null }) }) });
    expect(rowOf(container, 'A-1').textContent).toContain('no transcript found');
  });

  it('keeps the plain columns for an agent without live data', async () => {
    const { container } = await opened({ 'A-1': agent({ name: 'old-style', item: 'I-4', branch: 'agent/lead-x', note: 'checking' }) });
    const text = rowOf(container, 'A-1').textContent;
    for (const part of ['old-style', 'working', 'sonnet', 'build', 'I-4', 'lead-x', 'checking']) expect(text).toContain(part);
    expect(text).not.toContain('turns');
  });

  it('sorts the working agents by start, oldest first', async () => {
    const { container } = await opened({
      'A-2': agent({ name: 'second', startedAt: ago(5 * MIN) }),
      'A-1': agent({ name: 'first', startedAt: ago(20 * MIN) }),
    });
    const ids = [...container.querySelectorAll('[data-agent-id]')].map((row) => row.getAttribute('data-agent-id'));
    expect(ids).toEqual(['A-1', 'A-2']);
  });
});
