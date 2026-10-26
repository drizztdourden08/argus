/* @layer renderer-app @kind component */
import { Flex, Status, Tag } from '@drizztdourden08/tessera';
import { threadName } from '../model';
import { useTestsShared } from '../behavior/useTestsShared';
import type { TestChipsProps } from '../tests-props.type';

const TestChips = (props: TestChipsProps) => {
  const { test } = props;
  const { group, threads, conflicted } = useTestsShared();
  return (
    <Flex gap="xs" wrap>
      {group !== 'item' && test.item && <Tag title={`Item: ${test.item}`}>{test.item}</Tag>}
      {group !== 'thread' && test.thread && <Tag title={`Thread: ${test.thread}`}>{threadName(threads, test.thread)}</Tag>}
      {test.mode === 'auto' && <Tag title="A thread can run this one itself; you can still post a result">auto</Tag>}
      {conflicted.has(test.id) && <Status tone="danger" dot>conflict</Status>}
      {test.archivedAt && <Status tone="neutral" dot>archived</Status>}
    </Flex>
  );
};

export { TestChips };
