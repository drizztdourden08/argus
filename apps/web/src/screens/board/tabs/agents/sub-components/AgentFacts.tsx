/* @layer renderer-app @kind component */
import { DateText, FactsPanel } from '@drizztdourden08/tessera';
import type { FactsPanelFact } from '@drizztdourden08/tessera';
import type { AgentFactsProps } from '../AgentsTab.type';
import { agentElapsed, isActive } from '../model';

const moment = (label: string, value: string | null | undefined): FactsPanelFact[] =>
  value ? [{ label, value: <DateText value={value} format="datetime" /> }] : [];

const text = (label: string, value: string | null | undefined, copyable = false, mono = false): FactsPanelFact[] =>
  value ? [{ label, value, copyable, mono }] : [];

const AgentFacts = (props: AgentFactsProps) => {
  const { agent, now } = props;
  const where = [...text('Branch', agent.branch, true, true), ...text('Worktree', agent.worktree, true, true)];
  const ids = [...text('Thread', agent.thread), ...text('Item', agent.item), ...text('Agent id', agent.claudeId, true, true)];
  const times = [
    ...moment('Started', agent.startedAt),
    ...moment('Updated', agent.updatedAt),
    ...moment('Finished', agent.finishedAt),
    { label: isActive(agent) ? 'Elapsed' : 'Duration', value: agentElapsed(agent, now) },
  ];
  return <FactsPanel label="Agent facts" layout="rows" groups={[ids, where, times].filter((group) => group.length > 0)} />;
};

export { AgentFacts };
