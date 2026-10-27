/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { WithId } from '../../../../../data/react';
import type { Agent } from '../../../../../data/agents';
import type { AgentsBodyProps } from '../AgentsTab.type';
import { EMPTY_ALL, EMPTY_RUNNING } from '../model';
import { AgentRow } from './AgentRow';
import { AgentsHead } from './AgentsHead';

const AgentsBody = (props: AgentsBodyProps) => {
  const { active, done, showFinished, runs, onOpen, onLog, now, actions, state } = props;
  if (active.length + done.length === 0) return <Text as="p" tone="muted" className="agents-empty">{EMPTY_ALL}</Text>;
  const row = (agent: WithId<Agent>) => (
    <AgentRow key={agent.id} id={agent.id} agent={agent} now={now} actions={actions} state={state} runs={runs.byAgent.get(agent.id) ?? []} onOpen={onOpen} onLog={onLog} />
  );
  return (
    <Box role="table" aria-label="Sub-agents" className="agents-table">
      <AgentsHead />
      {active.length > 0 ? active.map(row) : <Text as="p" tone="muted" className="agents-empty">{EMPTY_RUNNING}</Text>}
      {showFinished && done.length > 0 && <Text as="p" variant="overline" className="agents-group">Finished</Text>}
      {showFinished && done.map(row)}
    </Box>
  );
};

export { AgentsBody };
