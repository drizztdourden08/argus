/* @layer renderer-app @kind component */
import { Flex, Status, Text } from '@drizztdourden08/tessera';
import { PR_STATUSES } from '../changes.constants';
import type { PrPartProps } from '../changes.type';

const PrHead = (props: PrPartProps) => {
  const { pr } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <Text variant="caption" tone="muted" mono>{pr.id}</Text>
      <Status map={PR_STATUSES} value={pr.status} dot size="sm" />
      <Text weight={600}>{pr.title}</Text>
    </Flex>
  );
};

export { PrHead };
