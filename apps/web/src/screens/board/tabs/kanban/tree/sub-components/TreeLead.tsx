/* @layer renderer-app @kind component */
import { Flex, Text, TypeMark } from '@drizztdourden08/tessera';
import type { IconName } from '@drizztdourden08/tessera';
import { itemTypeOf } from '../../../../../../data/registry';
import { categoryColor } from '../../model';
import type { TreeRowProps } from '../tree.type';

const TreeLead = (props: TreeRowProps) => {
  const { node } = props;
  const type = itemTypeOf(node.typeId);
  return (
    <Flex gap="xs" align="center">
      {type && <TypeMark icon={type.icon as IconName} label={type.label} color={categoryColor(type.color)} size="xs" />}
      <Text as="span" variant="caption" weight={600} mono>{node.id}</Text>
    </Flex>
  );
};

export { TreeLead };
