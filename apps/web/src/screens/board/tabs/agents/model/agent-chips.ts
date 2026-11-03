/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { RowChip } from '../../../../../shell/records';
import { agentElapsed } from './agent-elapsed';
import { agentWhere } from './agent-where';
import { modelLabel } from './model-label';
import { modelTip } from './model-tip';

const agentChips = (agent: Agent, now: number): RowChip[] => {
  const where = agentWhere(agent);
  const chips: RowChip[] = [{ id: 'where', text: where.text, title: where.tip }];
  if (agent.model) chips.push({ id: 'model', text: modelLabel(agent.model), title: modelTip(agent) });
  const elapsed = agentElapsed(agent, now);
  if (elapsed !== '-') chips.push({ id: 'elapsed', text: elapsed, title: 'Time since it started' });
  return chips;
};

export { agentChips };
