/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { AgentCellProps } from '../row-props.type';

const AgentCell = (props: AgentCellProps) => {
  const { name, title, children } = props;
  return (
    <Box role="cell" className={`agents-cell agents-cell--${name}`} title={title}>
      {children}
    </Box>
  );
};

export { AgentCell };
