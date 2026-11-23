/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { RecordRow } from '../../../../../shell';
import { agentHead } from '../behavior/agent-head';
import { agentChips, agentTitle } from '../model';
import type { AgentRowProps } from '../row-props.type';
import { AgentDetails } from './AgentDetails';
import { AgentTrailing } from './AgentTrailing';

const AgentRow = (props: AgentRowProps) => {
  const { id, agent, now, actions, onOpen, open, onRowOpen } = props;
  return (
    <Box data-swap-item>
      <RecordRow
        head={agentHead(props)}
        title={agentTitle(agent, id)}
        chips={agentChips(agent, now)}
        trailing={<AgentTrailing id={id} agent={agent} now={now} actions={actions} />}
        body={<AgentDetails agent={agent} now={now} />}
        onOpen={() => onOpen(id)}
        open={open.has(id)}
        onOpenChange={(value) => onRowOpen(id, value)}
      />
    </Box>
  );
};

export { AgentRow };
