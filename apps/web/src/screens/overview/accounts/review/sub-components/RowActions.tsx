/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { RowActionsProps } from '../review-tree-props.type';
import { ChatActions } from './ChatActions';
import { FolderActions } from './FolderActions';

const RowActions = (props: RowActionsProps) => (
  <Box className="review-tree-actions">
    {props.row.kind === 'folder' ? <FolderActions {...props} /> : <ChatActions {...props} />}
  </Box>
);

export { RowActions };
