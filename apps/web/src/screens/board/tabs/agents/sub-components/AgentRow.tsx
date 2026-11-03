/* @layer renderer-app @kind component */
import { RecordRow, recordMenuGroups } from '../../../../../shell';
import { agentHandlers } from '../behavior/agent-handlers';
import { agentChips, agentTitle } from '../model';
import type { AgentRowProps } from '../row-props.type';
import { AgentTrailing } from './AgentTrailing';

const AgentRow = (props: AgentRowProps) => {
  const { boardKey, id, agent, threads, now, actions, onOpen } = props;
  return (
    <RecordRow
      head={{
        kind: 'agent',
        code: id,
        assignee: agent.thread ?? '',
        boardKey,
        threads,
        menu: recordMenuGroups('agent', id, agentHandlers(id, agent, actions, onOpen)),
        menuLabel: `More actions for ${id}`,
      }}
      title={agentTitle(agent, id)}
      chips={agentChips(agent, now)}
      trailing={<AgentTrailing id={id} agent={agent} now={now} actions={actions} />}
      onOpen={() => onOpen(id)}
    />
  );
};

export { AgentRow };
