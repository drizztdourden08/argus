/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { FrameSlot, TypeIcon } from '../../../../../shell';
import type { TreeRowProps } from '../tree.type';

const TreeLead = (props: TreeRowProps) => {
  const { node } = props;
  return (
    <Flex gap="sm" align="center">
      <FrameSlot><TypeIcon kind={node.typeId} /></FrameSlot>
      <FrameSlot><Text as="span" variant="caption" leading="16" weight={600} mono>{node.id}</Text></FrameSlot>
    </Flex>
  );
};

export { TreeLead };
