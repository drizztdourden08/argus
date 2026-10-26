/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import type { TestChipsProps } from '../tests-props.type';

const TestChips = (props: TestChipsProps) => {
  const { test } = props;
  return (
    <Flex gap="xs" wrap>
      {test.item && <Tag title={`Item: ${test.item}`}>{test.item}</Tag>}
      {test.thread && <Tag title={`Thread: ${test.thread}`}>{test.thread}</Tag>}
      {test.mode === 'auto' && <Tag title="A thread can run this one itself; you can still post a result">auto</Tag>}
    </Flex>
  );
};

export { TestChips };
