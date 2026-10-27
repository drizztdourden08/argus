/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { COLUMNS } from '../AgentsTab.constants';

const AgentsHead = () => (
  <Box role="row" className="agents-row agents-row--head">
    {COLUMNS.map((column) => (
      <Box key={column.id} role="columnheader" className={`agents-cell agents-cell--${column.id}`} aria-label={column.label}>
        {column.id === 'actions' ? null : column.label}
      </Box>
    ))}
  </Box>
);

export { AgentsHead };
