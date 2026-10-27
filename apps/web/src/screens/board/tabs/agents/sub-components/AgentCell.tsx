/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { AgentCellProps } from '../row-props.type';

const AgentCell = (props: AgentCellProps) => {
  const { name, title, agentId, children } = props;
  return (
    <Box className={`agents-cell agents-cell--${name}`} title={title} data-agent-id={agentId}>
      {children}
    </Box>
  );
};

export { AgentCell };
