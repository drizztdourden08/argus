/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { NONE_LABELS } from './agents-view.constants';
import type { KeyGroup } from './agents-view.type';
import type { KeyedGroup } from './keyed-group.type';
import { modelFamily } from './model-family';
import { statusLook } from './status-look';

const keyOf = (agent: Agent, group: KeyedGroup): string => {
  if (group === 'thread') return agent.thread ?? '';
  if (group === 'status') return agent.status;
  return agent.model ? modelFamily(agent.model) : '';
};

const labelOf = (key: string, group: KeyedGroup): string => {
  if (!key) return NONE_LABELS[group];
  if (group === 'status') return statusLook(key).label;
  return group === 'model' ? `${key.charAt(0).toUpperCase()}${key.slice(1)}` : key;
};

const byLabel = (a: KeyGroup, b: KeyGroup): number => {
  if (!a.key !== !b.key) return a.key ? -1 : 1;
  return a.label.localeCompare(b.label, undefined, { numeric: true });
};

const groupAgentsBy = (agents: readonly WithId<Agent>[], group: KeyedGroup): KeyGroup[] => {
  const found = new Map<string, WithId<Agent>[]>();
  for (const agent of agents) {
    const key = keyOf(agent, group);
    found.set(key, [...(found.get(key) ?? []), agent]);
  }
  return [...found].map(([key, rows]) => ({ key, label: labelOf(key, group), rows })).sort(byLabel);
};

export { groupAgentsBy };
