/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { agentDetails } from '../model';
import type { AgentDetailsProps } from '../row-props.type';

const AgentDetails = (props: AgentDetailsProps) => {
  const { agent, now } = props;
  const lines = agentDetails(agent, now);
  if (lines.length === 0) return null;
  return (
    <Box as="dl" className="agents-kv agents-details">
      {lines.map((line) => (
        <Box key={line.id} className="agents-details__line">
          <Box as="dt" className="agents-kv__label">{line.label}</Box>
          <Box as="dd">{line.value}</Box>
        </Box>
      ))}
    </Box>
  );
};

export { AgentDetails };
