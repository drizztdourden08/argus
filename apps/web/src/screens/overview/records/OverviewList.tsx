/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { OverviewListProps } from './records.type';

const OverviewList = (props: OverviewListProps) => (
  <Flex direction="column" gap="2xs" role="list" aria-label={props.label}>
    {props.children}
  </Flex>
);

export { OverviewList };
