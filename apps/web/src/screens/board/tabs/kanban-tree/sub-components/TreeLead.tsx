/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { TypeIcon } from '../../../../../shell';
import type { TreeRowProps } from '../tree.type';

const TreeLead = (props: TreeRowProps) => {
  const { node } = props;
  return (
    <Flex gap="xs" align="center">
      <TypeIcon kind={node.typeId} />
      <Text as="span" variant="caption" weight={600} mono>{node.id}</Text>
    </Flex>
  );
};

export { TreeLead };
