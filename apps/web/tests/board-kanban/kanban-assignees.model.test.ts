/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { agentTone, assigneeLook, itemAgents, withItemAgents } from '../../src/screens/board/tabs/kanban/model';
import { assigneeChoices } from '../../src/screens/board/tabs/kanban/model/assignee-choices';
import type { AgentDoc } from '../../src/screens/board/tabs/kanban/kanban.type';
import { itemRef, threadRef } from './kanban-docs';

const agent = (id: string, extra: Partial<AgentDoc> = {}): AgentDoc => ({ id, name: `agent ${id}`, status: 'working', ...extra });

describe('I-168 readable assignee names', { tags: ['critical'] }, () => {
  const threads = [threadRef('t-1', { session: 'Builder' }), threadRef('t-2', { session: '', name: 'Fixer' }), threadRef('t-3')];

  it('names a thread by its session, then its name, then its id', () => {
    expect(assigneeLook('t-1', 'argus', threads).label).toBe('Builder');
    expect(assigneeLook('t-1', 'argus', threads).tip).toBe('Assigned to Builder');
    expect(assigneeLook('t-2', 'argus', threads).label).toBe('Fixer');
    expect(assigneeLook('t-9', 'argus', threads).label).toBe('t-9');
  });

  it('labels a lead with the board name and falls back to the key', () => {
    expect(assigneeLook('lead', 'b5qa9b7', threads).label).toBe('b5qa9b7 lead');
    expect(assigneeLook('lead:brock', 'argus', threads).tip).toBe('Assigned to the lead of brock');
  });
});

describe('I-167 agents as assignees and on tickets', { tags: ['critical'] }, () => {
  const agents = [agent('A-1', { item: 'I-1' }), agent('A-2', { items: ['I-1', 'I-2'] }), agent('A-3', { item: 'I-1', status: 'done' })];

  it('renders agent:A-n as the agent name, or the id when unknown', () => {
    expect(assigneeLook('agent:A-1', 'argus', [], undefined, agents).label).toBe('agent A-1');
    expect(assigneeLook('agent:A-7', 'argus', [], undefined, agents).label).toBe('A-7');
  });

  it('offers the live agents as agent:A-n choices', () => {
    const values = assigneeChoices([itemRef('I-1')], 'argus', [], agents).map((option) => option.value);
    expect(values).toEqual(['owner', 'worker', 'agent:A-1', 'agent:A-2']);
  });

  it('finds the live agents of an item from item or items', () => {
    expect(itemAgents(agents, 'I-1').map((found) => found.id)).toEqual(['A-1', 'A-2']);
    expect(itemAgents(agents, 'I-2').map((found) => found.id)).toEqual(['A-2']);
  });

  it('fills the agents of an item and keeps the ones it already has', () => {
    const [first, second] = withItemAgents([itemRef('I-1'), itemRef('I-9', { agents: [{ id: 'A-5', name: 'own' }] })], agents);
    expect(first?.agents?.map((found) => found.id)).toEqual(['A-1', 'A-2']);
    expect(second?.agents).toEqual([{ id: 'A-5', name: 'own' }]);
  });

  it('maps an agent state to a tone', () => {
    expect(agentTone('working')).toBe('success');
    expect(agentTone('failed')).toBe('danger');
    expect(agentTone(undefined)).toBe('info');
  });
});
