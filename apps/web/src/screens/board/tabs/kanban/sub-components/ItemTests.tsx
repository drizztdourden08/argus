/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import type { ItemTestsProps } from '../drawer-props.type';
import { DrawerSection } from './DrawerSection';

const ItemTests = (props: ItemTestsProps) => {
  const { tests } = props;
  const passing = tests.filter((test) => test.status === 'pass').length;
  return (
    <DrawerSection title={`Tests, ${passing}/${tests.length} passing`}>
      {tests.length === 0 && <Text tone="muted">No test documented for this item.</Text>}
      <Flex direction="column" gap="xs">
        {tests.map((test) => (
          <Flex key={test.id} gap="sm" align="center" wrap>
            <Text variant="caption" tone="muted" mono>{test.id}</Text>
            <Tag>{test.status}</Tag>
            <Text>{test.title}</Text>
          </Flex>
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { ItemTests };
