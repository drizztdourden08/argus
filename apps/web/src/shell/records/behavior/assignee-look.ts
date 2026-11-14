/* @layer renderer-app @kind logic */
import { OWNER_DEFAULTS } from '../../../owner';
import type { OwnerProfile } from '../../../owner';
import { boardName } from '../../../data/boards';
import type { ThreadChoice } from '../../../data/kanban';
import type { WithId } from '../../../data/react';
import { ITEM_REGISTRY } from '../../../data/registry';
import { categoryColor } from './category-color';
import { AGENT_PREFIX, COORDINATOR, LEAD_PREFIX, OWNER } from './look.constants';
import type { AgentName, AssigneeLook } from '../records.type';

type Threads = readonly WithId<ThreadChoice>[];

const leadLook = (key: string, tip: (name: string) => string): AssigneeLook => {
  const look = ITEM_REGISTRY.people.leads[key];
  const name = boardName(key);
  return { label: `${name} lead`, tip: tip(name), initials: look?.initials ?? name.slice(0, 2).toUpperCase(), color: categoryColor(look?.color), lead: key, owner: false };
};

const isLead = (assignee: string, threads: Threads): boolean =>
  assignee === COORDINATOR || threads.some((thread) => thread.id === assignee && thread.role === COORDINATOR);

const trimmed = (value: string | null | undefined): string => value?.trim() ?? '';

const threadName = (assignee: string, threads: Threads): string => {
  const thread = threads.find((candidate) => candidate.id === assignee);
  return trimmed(thread?.session) || trimmed(thread?.name) || assignee;
};

const agentLook = (id: string, agents: readonly AgentName[]): AssigneeLook => {
  const name = trimmed(agents.find((agent) => agent.id === id)?.name) || id;
  return { label: name, tip: `Assigned to agent ${name}`, initials: name.slice(0, 2).toUpperCase(), owner: false };
};

const assigneeLook = (assignee: string, boardKey: string, threads: Threads, owner: OwnerProfile = OWNER_DEFAULTS, agents: readonly AgentName[] = []): AssigneeLook => {
  if (assignee === OWNER) return { label: owner.name, tip: 'Assigned to you', icon: 'user', owner: true };
  if (assignee.startsWith(AGENT_PREFIX)) return agentLook(assignee.slice(AGENT_PREFIX.length), agents);
  if (assignee.startsWith(LEAD_PREFIX)) return leadLook(assignee.slice(LEAD_PREFIX.length), (name) => `Assigned to the lead of ${name}`);
  if (isLead(assignee, threads)) return leadLook(boardKey, (name) => `Assigned to ${name} lead (coordinator)`);
  const label = threadName(assignee, threads);
  return { label, tip: `Assigned to ${label}`, initials: label.slice(0, 2).toUpperCase(), owner: false };
};

export { assigneeLook };
