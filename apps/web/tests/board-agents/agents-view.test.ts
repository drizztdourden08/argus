/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { Agent } from '../../src/data/agents';
import type { WithId } from '../../src/data/react';
import { AGENT_FILTERS, agentsView, groupAgentsBy } from '../../src/screens/board/tabs/agents/model';

const agent = (id: string, extra: Partial<Agent> = {}): WithId<Agent> => ({ id, status: 'working', ...extra }) as WithId<Agent>;

const ids = (list: WithId<Agent>[]): string[] => list.map((entry) => entry.id);

describe('I-105 agents filter, sort and group', { tags: ['critical'] }, () => {
  const base = {
    active: [agent('A-1', { name: 'beta', thread: 'x', model: 'claude-haiku-5' }), agent('A-2', { name: 'alpha', thread: 'y', model: 'claude-opus-5', note: 'fix login' })],
    done: [],
  };

  it('keeps today\'s order for Updated desc and reverses it for asc', () => {
    expect(ids(agentsView(base, { filters: AGENT_FILTERS, order: 'updated', direction: 'desc' }).active)).toEqual(['A-1', 'A-2']);
    expect(ids(agentsView(base, { filters: AGENT_FILTERS, order: 'updated', direction: 'asc' }).active)).toEqual(['A-2', 'A-1']);
  });

  it('sorts by name and filters by search, thread and model', () => {
    expect(ids(agentsView(base, { filters: AGENT_FILTERS, order: 'name', direction: 'asc' }).active)).toEqual(['A-2', 'A-1']);
    expect(ids(agentsView(base, { filters: { ...AGENT_FILTERS, query: 'login' }, order: 'name', direction: 'asc' }).active)).toEqual(['A-2']);
    expect(ids(agentsView(base, { filters: { ...AGENT_FILTERS, thread: ['x'] }, order: 'name', direction: 'asc' }).active)).toEqual(['A-1']);
    expect(ids(agentsView(base, { filters: { ...AGENT_FILTERS, model: ['opus'] }, order: 'name', direction: 'asc' }).active)).toEqual(['A-2']);
  });

  it('groups by thread with the unthreaded last', () => {
    const rows = [agent('A-1', { thread: 'b' }), agent('A-2'), agent('A-3', { thread: 'a' })];
    expect(groupAgentsBy(rows, 'thread').map((group) => group.label)).toEqual(['a', 'b', 'No thread']);
  });
});
