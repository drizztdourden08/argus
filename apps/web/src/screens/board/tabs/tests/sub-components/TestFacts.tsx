/* @layer renderer-app @kind component */
import { DateText, Flex, Text } from '@drizztdourden08/tessera';
import type { TestFactsProps } from '../tests-drawer-props.type';
import { FactTags } from './FactTags';

const TestFacts = (props: TestFactsProps) => {
  const { test } = props;
  const editor = test.updatedBy ?? test.createdBy ?? 'unknown';
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="sm" align="center" wrap>
        <FactTags {...props} />
        <Text variant="caption" tone="muted">updated <DateText value={test.updatedAt} format="relative" /></Text>
      </Flex>
      <Text variant="caption" tone="faint">{`rev ${test.rev ?? 1}, updated by ${editor}`}</Text>
    </Flex>
  );
};

export { TestFacts };
