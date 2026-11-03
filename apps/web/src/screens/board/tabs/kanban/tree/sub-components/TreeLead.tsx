/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { HEAD_ID_CLASS } from '../../head.constants';
import { TypeIcon } from '../../sub-components/TypeIcon';
import type { TreeRowProps } from '../tree.type';

const TreeLead = (props: TreeRowProps) => {
  const { node } = props;
  return (
    <Flex gap="xs" align="center">
      <TypeIcon typeId={node.typeId} />
      <Text as="span" variant="caption" weight={600} mono className={HEAD_ID_CLASS}>{node.id}</Text>
    </Flex>
  );
};

export { TreeLead };
