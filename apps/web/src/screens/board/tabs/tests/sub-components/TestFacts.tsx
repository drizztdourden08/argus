/* @layer renderer-app @kind component */
import { DateText, Flex, Status, Tag, Text } from '@drizztdourden08/tessera';
import { TEST_STATUSES } from '../tests.constants';
import type { TestFactsProps } from '../tests-props.type';

const TestFacts = (props: TestFactsProps) => {
  const { test, handed } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <Status map={TEST_STATUSES} value={test.status} variant="pill" dot />
      <Tag color={test.mode === 'manual' ? 'primary' : 'neutral'}>{test.mode === 'manual' ? 'manual: needs you' : 'auto'}</Tag>
      {test.item && <Tag title={`Item ${test.item}`}>{test.item}</Tag>}
      {test.thread && <Tag title={`Thread ${test.thread}`}>{test.thread}</Tag>}
      {test.build && <Tag title="The build this test runs on">{test.build}</Tag>}
      {handed && <Tag color="primary">handed to you</Tag>}
      <Text variant="caption" tone="muted">updated <DateText value={test.updatedAt} format="relative" /></Text>
    </Flex>
  );
};

export { TestFacts };
