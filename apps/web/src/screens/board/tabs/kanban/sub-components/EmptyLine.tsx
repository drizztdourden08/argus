/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { EmptyLineProps } from '../drawer-props.type';

const EmptyLine = (props: EmptyLineProps) => (
  <Flex justify="center">
    <Text variant="caption" tone="muted">{props.children}</Text>
  </Flex>
);

export { EmptyLine };
