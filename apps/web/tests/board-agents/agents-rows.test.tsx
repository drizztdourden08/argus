/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { agent, agentRows, item, mockApi, renderAgents, rowOf } from './agents-harness';

afterEach(() => {
  localStorage.clear();
  cleanup();
  vi.unstubAllGlobals();
});

const where = (): string => screen.getByLabelText('where').textContent;

describe('I-85 P7 Agents tab: rows grouped by item', { tags: ['critical'] }, () => {
  it('groups the agents under the item they work on, with the item as the header row', async () => {
    mockApi();
    const { container } = await renderAgents({
      agents: { 'A-1': agent({ items: ['I-1'] }), 'A-2': agent({ items: ['I-2'] }), 'A-3': agent({ items: ['I-1'] }) },
      items: { 'I-1': item({ title: 'first job', rank: 2 }), 'I-2': item({ title: 'second job', rank: 1 }) },
    });
    expect(rowOf(container, 'I-1').textContent).toContain('first job');
    expect(rowOf(container, 'I-2').textContent).toContain('second job');
    const heads = [...container.querySelectorAll<HTMLElement>('[data-record^="I-"]')].map((row) => row.dataset.record);
    expect(heads).toEqual(['I-2', 'I-1']);
    expect(agentRows(container)).toEqual(['A-2', 'A-1', 'A-3']);
  });

  it('lists an agent on two items under both', async () => {
    mockApi();
    const { container } = await renderAgents({
      agents: { 'A-1': agent({ items: ['I-1', 'I-2'] }) },
      items: { 'I-1': item(), 'I-2': item({ title: 'other' }) },
    });
    expect(agentRows(container)).toEqual(['A-1', 'A-1']);
  });

  it('flags the agents that have no item', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: { 'A-1': agent() }, items: {} });
    expect(screen.getByText('No item')).toBeTruthy();
    expect(screen.getByText('no link')).toBeTruthy();
    expect(agentRows(container)).toEqual(['A-1']);
  });

  it('folds the agents of a closed item', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: { 'A-1': agent({ items: ['I-1'] }) }, items: { 'I-1': item({ status: 'done' }) } });
    expect(screen.getByText('Closed items')).toBeTruthy();
    expect(container.querySelectorAll('[data-record="A-1"]').length).toBeLessThanOrEqual(1);
  });
});

describe('I-85 P7 Agents tab: the where chip', { tags: ['critical'] }, () => {
  const chipsOf = async (extra: Record<string, unknown>) => {
    mockApi();
    const { container } = await renderAgents({ agents: { 'A-1': agent({ items: ['I-1'], ...extra }) }, items: { 'I-1': item() } });
    return within(rowOf(container, 'A-1'));
  };

  it('names the worktree folder', async () => {
    expect((await chipsOf({ where: 'X:/p/trees/agent-ab12', whereSource: 'manual' })).getByText('WT agent-ab12')).toBeTruthy();
  });

  it('says master for an agent on the main checkout', async () => {
    expect((await chipsOf({ master: true, where: 'master', whereSource: 'manual' })).getByText('master')).toBeTruthy();
  });

  it('says no worktree when the place is unknown', async () => {
    expect((await chipsOf({})).getByText('no worktree')).toBeTruthy();
  });
});

describe('I-85 P7 Agents tab: the pane', { tags: ['critical'] }, () => {
  it('opens the agent pane over the tab when the row is clicked', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: { 'A-2': agent({ items: ['I-1'] }) }, items: { 'I-1': item() } });
    fireEvent.click(within(rowOf(container, 'A-2')).getByText('worker: do it'));
    expect(where()).toBe('?open=A-2');
  });

  it('opens the item pane from the item header', async () => {
    mockApi();
    const { container } = await renderAgents({ agents: { 'A-2': agent({ items: ['I-1'] }) }, items: { 'I-1': item() } });
    fireEvent.click(within(rowOf(container, 'I-1')).getByText('ship it'));
    expect(where()).toBe('?open=I-1');
  });
});
