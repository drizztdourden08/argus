/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { TreeStepsProps } from '../review-tree-props.type';

const TreeSteps = (props: TreeStepsProps) => (
  <>
    {Array.from({ length: props.depth }, (_, level) => <Box key={level} as="span" className="review-tree-step" aria-hidden="true" />)}
  </>
);

export { TreeSteps };
