/* @layer renderer-app @kind test */
import { toast } from '@drizztdourden08/tessera';
import { act, cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { agent, ago, json, MIN, mockApi, mockFetch, renderAgents, rowOf } from './agents-harness';

afterEach(() => {
  act(() => toast.clear());
  cleanup();
  vi.unstubAllGlobals();
});

const stopButton = (row: HTMLElement) => within(row).getByRole('button', { name: 'Ask the lead to stop this agent' });

describe('Agents tab: the owner stop request', () => {
  it('posts an edit entry that sets stop-requested, and tells the owner', async () => {
    const posted = mockApi();
    const { container } = await renderAgents({ agents: { 'A-1': agent({ rev: 3 }) } });
    fireEvent.click(stopButton(rowOf(container, 'A-1')));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.url).toBe('/api/p/alpha/entries');
    expect(posted[0]?.body).toMatchObject({
      kind: 'edit',
      author: 'human',
      target: { c: 'agents', id: 'A-1' },
      set: { status: 'stop-requested' },
      prev: { status: 'working' },
      baseRev: 3,
    });
    expect(String(posted[0]?.body.id)).toMatch(/^O-/);
    expect(await screen.findByText('Stop requested for worker. The lead is told at its next check.')).toBeTruthy();
  });

  it('offers Undo on that toast, and the undo posts the status back with the id it undoes', async () => {
    const posted = mockApi();
    const { container } = await renderAgents({ agents: { 'A-1': agent() } });
    fireEvent.click(stopButton(rowOf(container, 'A-1')));
    fireEvent.click(await screen.findByRole('button', { name: /Undo/ }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'edit', set: { status: 'working' }, undoes: posted[0]?.body.id });
  });

  it('shows a stopping agent with a button that cancels the request', async () => {
    const posted = mockApi();
    const { container } = await renderAgents({ agents: { 'A-1': agent({ status: 'stop-requested', rev: 4 }) } });
    const row = rowOf(container, 'A-1');
    expect(row.textContent).toContain('stopping');
    expect(within(row).queryByRole('button', { name: 'Ask the lead to stop this agent' })).toBeNull();
    fireEvent.click(within(row).getByRole('button', { name: 'Cancel the stop request' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ set: { status: 'working' }, prev: { status: 'stop-requested' }, baseRev: 4 });
    expect(await screen.findByText('Stop request cancelled for worker.')).toBeTruthy();
  });

  it('counts the working, waiting and stopping agents in the header', async () => {
    mockApi();
    await renderAgents({
      agents: {
        'A-1': agent(),
        'A-2': agent({ status: 'waiting' }),
        'A-3': agent({ status: 'stop-requested' }),
      },
    });
    expect(screen.getByText('working 1, waiting 1, stopping 1')).toBeTruthy();
  });

  it('reports a conflict from the board', async () => {
    mockFetch((_url, init) => (
      init?.method === 'POST'
        ? json({ id: 'x', status: 'conflict', outcome: 'a thread edited it', entry: null, seq: 1 })
        : json({ ran: true, at: null, findings: [] })
    ));
    const { container } = await renderAgents({ agents: { 'A-1': agent() } });
    fireEvent.click(stopButton(rowOf(container, 'A-1')));
    expect(await screen.findByText('Conflict: a thread edited it. Nothing of yours was applied.')).toBeTruthy();
  });
});

describe('Agents tab: filters and archive', () => {
  const board = {
    'A-1': agent({ name: 'alpha-worker' }),
    'A-2': agent({ name: 'zeta-worker', task: 'zeta job' }),
    'A-3': agent({ name: 'finished', status: 'done', finishedAt: ago(MIN), result: 'ok' }),
    'A-4': agent({ name: 'hidden', status: 'done', archivedAt: ago(MIN) }),
  };

  it('shows the running agents first, the finished ones under Done, and all of them under All', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: board });
    expect(screen.getByRole('radio', { name: 'Running (2)' })).toBeTruthy();
    expect(screen.getByRole('radio', { name: 'Done (1)' })).toBeTruthy();
    expect(container.querySelector('[data-agent-id="A-3"]')).toBeNull();
    fireEvent.click(screen.getByRole('radio', { name: 'Done (1)' }));
    expect(container.querySelector('[data-agent-id="A-3"]')).not.toBeNull();
    expect(container.querySelector('[data-agent-id="A-1"]')).toBeNull();
    fireEvent.click(screen.getByRole('radio', { name: 'All' }));
    expect(container.querySelectorAll('[data-agent-id]')).toHaveLength(3);
  });

  it('never lists an archived agent', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: board });
    fireEvent.click(screen.getByRole('radio', { name: 'All' }));
    expect(container.querySelector('[data-agent-id="A-4"]')).toBeNull();
  });

  it('searches by name, task or item', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: board });
    fireEvent.change(screen.getByRole('searchbox', { name: 'Search agents' }), { target: { value: 'zeta job' } });
    expect([...container.querySelectorAll('[data-agent-id]')].map((row) => row.getAttribute('data-agent-id'))).toEqual(['A-2']);
  });

  it('tells the owner when nothing runs and when no agent exists', async () => {
    mockApi();
    const first = await renderAgents({ agents: { 'A-3': board['A-3'] } });
    expect(screen.getByText('No sub-agent is working right now.')).toBeTruthy();
    first.unmount();
    mockApi();
    await renderAgents({ agents: {} });
    expect(screen.getByText('No sub-agent yet. The lead starts them as it works and they show here.')).toBeTruthy();
  });

  it('archives the finished agents in one bulk entry, with Undo', async () => {
    const posted = mockApi();
    await renderAgents({ agents: board });
    fireEvent.click(screen.getByRole('radio', { name: 'Done (1)' }));
    fireEvent.click(screen.getByRole('button', { name: 'Archive finished' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'bulk', ops: [{ kind: 'archive', target: { c: 'agents', id: 'A-3' } }] });
    expect(await screen.findByText('1 finished agent archived.')).toBeTruthy();
    fireEvent.click(await screen.findByRole('button', { name: /Undo/ }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'bulk', ops: [{ kind: 'unarchive', target: { c: 'agents', id: 'A-3' } }], undoes: posted[0]?.body.id });
  });

  it('offers no archive button on the Running view', async () => {
    mockApi();
    await renderAgents({ agents: board });
    expect(screen.queryByRole('button', { name: 'Archive finished' })).toBeNull();
  });
});
