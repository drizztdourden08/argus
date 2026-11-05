/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { FrameCodeProps } from '../record-frame.type';

const FrameCode = (props: FrameCodeProps) => (
  <Text as="span" variant="caption" leading="16" tone="muted" mono>{props.children}</Text>
);

export { FrameCode };
