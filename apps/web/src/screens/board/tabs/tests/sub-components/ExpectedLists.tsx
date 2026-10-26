/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { TestPartProps } from '../tests-props.type';
import { DrawerSection } from './DrawerSection';
import { TextList } from './TextList';

const ExpectedLists = (props: TestPartProps) => {
  const { test } = props;
  if (test.pass.length === 0 && test.fail.length === 0) return null;
  return (
    <DrawerSection title="Expected">
      <Flex gap="lg" wrap>
        <Flex direction="column" gap="2xs">
          <Text variant="label" tone="success">Proves it works</Text>
          <TextList lines={test.pass} label="Proves it works" />
        </Flex>
        <Flex direction="column" gap="2xs">
          <Text variant="label" tone="danger">Proves it is broken</Text>
          <TextList lines={test.fail} label="Proves it is broken" />
        </Flex>
      </Flex>
    </DrawerSection>
  );
};

export { ExpectedLists };
