/* @layer renderer-app @kind component */
import { Card, Flex, Pressable, Text } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import type { ItemTestsProps } from '../drawer-props.type';
import { DrawerSection } from './DrawerSection';
import { EmptyLine } from './EmptyLine';
import { StatusTag } from './StatusTag';

const ItemTests = (props: ItemTestsProps) => {
  const { tests } = props;
  const navigate = useNavigate();
  const passing = tests.filter((test) => test.status === 'pass').length;
  return (
    <DrawerSection title={`Tests · ${passing}/${tests.length} passing`}>
      {tests.length === 0 && <EmptyLine>No test documented for this item.</EmptyLine>}
      <Flex direction="column" gap="xs">
        {tests.map((test) => (
          <Card key={test.id} variant="interactive">
            <Pressable aria-label={`Open ${test.id}: ${test.title}`} onClick={() => void navigate({ hash: test.id })}>
              <Flex direction="column" gap="xs">
                <Flex gap="xs" align="center" wrap>
                  <Text variant="caption" weight={600} mono>{test.id}</Text>
                  <StatusTag status={test.status} />
                  <StatusTag status={test.mode} />
                </Flex>
                <Text>{test.title}</Text>
              </Flex>
            </Pressable>
          </Card>
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { ItemTests };
