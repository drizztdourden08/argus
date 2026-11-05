/* @layer renderer-app @kind component */
import { Flex, Pressable, Text } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { RecordFrame } from '../../../../../shell';
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
          <RecordFrame
            key={test.id}
            name={test.id}
            density="row"
            code={test.id}
            title={(
              <Pressable aria-label={`Open ${test.id}: ${test.title}`} onClick={() => void navigate({ hash: test.id })}>
                <Text smartTimes>{test.title}</Text>
              </Pressable>
            )}
            content={(
              <Flex gap="xs" align="center">
                <StatusTag status={test.status} />
                <StatusTag status={test.mode} />
              </Flex>
            )}
          />
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { ItemTests };
