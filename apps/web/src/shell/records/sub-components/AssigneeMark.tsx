/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { AssigneeMarkProps } from '../records.type';
import { AssigneeFace } from './AssigneeFace';
import './FrameMark.css';

const AssigneeMark = (props: AssigneeMarkProps) => (
  <Box as="span" className="frame-mark">
    <AssigneeFace {...props} />
  </Box>
);

export { AssigneeMark };
