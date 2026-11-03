/* @layer renderer-app @kind test */
import { toast } from '@drizztdourden08/tessera';
import { act, cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { agent, ago, json, MIN, mockApi, mockFetch, renderAgents, rowOf } from './agents-harness';

afterEach(() => {
  localStorage.clear();
  act(() => toast.clear());
  cleanup();
  vi.unstubAllGlobals();
});

const stopButton = (row: HTMLElement) => within(row).getByRole('button', { name: 'Ask the lead to stop this agent' });

describe('S1 Agents tab: the owner stop request', { tags: ['standard'] }, () => {
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
    expect(screen.getByText('working 1 \u00b7 waiting 1 \u00b7 stopping 1')).toBeTruthy();
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

describe('S1 Agents tab: the finished agents and archive', { tags: ['standard'] }, () => {
  const board = {
    'A-1': agent({ name: 'alpha-worker' }),
    'A-2': agent({ name: 'zeta-worker', task: 'zeta job' }),
    'A-3': agent({ name: 'finished', status: 'done', finishedAt: ago(MIN), result: 'ok' }),
    'A-4': agent({ name: 'hidden', status: 'done', archivedAt: ago(MIN) }),
  };

  it('shows the running agents first and the finished ones only under Show finished', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: board });
    expect(screen.getByRole('button', { name: 'Show finished (1)' })).toBeTruthy();
    expect(container.querySelector('[data-agent-id="A-3"]')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Show finished (1)' }));
    expect(container.querySelector('[data-agent-id="A-3"]')).not.toBeNull();
    expect(container.querySelectorAll('[data-agent-id]')).toHaveLength(3);
    fireEvent.click(screen.getByRole('button', { name: 'Hide finished (1)' }));
    expect(container.querySelector('[data-agent-id="A-3"]')).toBeNull();
  });

  it('never lists an archived agent', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: board });
    fireEvent.click(screen.getByRole('button', { name: 'Show finished (1)' }));
    expect(container.querySelector('[data-agent-id="A-4"]')).toBeNull();
  });

  it('archives the finished agents in one bulk entry, with Undo', async () => {
    const posted = mockApi();
    await renderAgents({ agents: board });
    fireEvent.click(screen.getByRole('button', { name: 'Show finished (1)' }));
    fireEvent.click(screen.getByRole('button', { name: 'Archive finished' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'bulk', ops: [{ kind: 'archive', target: { c: 'agents', id: 'A-3' } }] });
    expect(await screen.findByText('1 finished agent archived.')).toBeTruthy();
    fireEvent.click(await screen.findByRole('button', { name: /Undo/ }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'bulk', ops: [{ kind: 'unarchive', target: { c: 'agents', id: 'A-3' } }], undoes: posted[0]?.body.id });
  });
});

describe('S1 Agents tab: the row actions', { tags: ['standard'] }, () => {
  const menuOf = (container: HTMLElement) => {
    vi.stubGlobal('CSS', { supports: () => false });
    fireEvent.click(within(rowOf(container, 'A-1')).getByRole('button', { name: 'More actions for A-1' }));
  };

  it('edits the note from the pencil and sends it as an edit with the old note and the revision', async () => {
    const posted = mockApi();
    const { container } = await renderAgents({ agents: { 'A-1': agent({ note: 'old note' }) } });
    fireEvent.click(within(rowOf(container, 'A-1')).getByRole('button', { name: 'Edit the note' }));
    const field = screen.getByRole('textbox', { name: 'Note' });
    fireEvent.change(field, { target: { value: 'new note' } });
    fireEvent.keyDown(field, { key: 'Enter' });
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'edit', target: { c: 'agents', id: 'A-1' }, set: { note: 'new note' }, prev: { note: 'old note' }, baseRev: 3 });
  });

  it('archives from the kebab', async () => {
    const posted = mockApi();
    const { container } = await renderAgents({ agents: { 'A-1': agent() } });
    menuOf(container);
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Archive' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'archive', target: { c: 'agents', id: 'A-1' } });
  });
});
