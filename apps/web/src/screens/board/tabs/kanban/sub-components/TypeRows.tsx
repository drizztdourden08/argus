/* @layer renderer-app @kind component */
import { Flex, Text, TypeMark } from '@drizztdourden08/tessera';
import type { IconName } from '@drizztdourden08/tessera';
import { ITEM_REGISTRY } from '../../../../../data/registry';
import { categoryColor } from '../model';

const TypeRows = () => (
  <Flex gap="md" align="center" wrap>
    {ITEM_REGISTRY.types.filter((type) => type.collection === 'items').map((type) => (
      <Flex key={type.id} gap="xs" align="center">
        <TypeMark icon={type.icon as IconName} label={type.label} color={categoryColor(type.color)} size="xs" />
        <Text variant="subtitle">{type.label}</Text>
      </Flex>
    ))}
  </Flex>
);

export { TypeRows };
