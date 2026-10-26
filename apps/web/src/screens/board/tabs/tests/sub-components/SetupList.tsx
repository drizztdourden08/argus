/* @layer renderer-app @kind component */
import { CopyValue, Flex, Markdown, Text } from '@drizztdourden08/tessera';
import type { TestPartProps } from '../tests-props.type';
import { DrawerSection } from './DrawerSection';

const SetupList = (props: TestPartProps) => {
  const { test } = props;
  if (test.setup.length === 0 && !test.run) return null;
  return (
    <DrawerSection title="Setup">
      {test.setup.map((entry) => (
        <Flex key={entry.label} gap="sm" align="baseline" wrap>
          <Text variant="label">{entry.label}</Text>
          <Markdown size="sm">{entry.text}</Markdown>
        </Flex>
      ))}
      {test.run && <CopyValue value={test.run} label="Run command" mono />}
    </DrawerSection>
  );
};

export { SetupList };
