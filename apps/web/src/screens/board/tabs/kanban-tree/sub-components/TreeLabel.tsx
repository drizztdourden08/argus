/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { TreeRowProps } from '../tree.type';

const TreeLabel = (props: TreeRowProps) => (
  <Text as="span" clamp={1} smartTimes>{props.node.title}</Text>
);

export { TreeLabel };
