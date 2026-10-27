/* @layer renderer-app @kind component */
import { Flex, Status, Tag } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { testChips } from '../model';
import type { TestChipsProps } from '../tests-props.type';

const TestChips = (props: TestChipsProps) => {
  const { group, conflicted } = useTestsShared();
  const chips = testChips(props.test, group, conflicted);
  if (!Object.values(chips).some(Boolean)) return null;
  return (
    <Flex gap="xs" align="center" wrap>
      {chips.item && <Tag title={`Item: ${chips.item}`}>{chips.item}</Tag>}
      {chips.thread && <Tag title={`Thread: ${chips.thread}`}>{chips.thread}</Tag>}
      {chips.auto && <Tag variant="urgency" color="info" title="A thread can run this one itself; you can still post a result">auto</Tag>}
      {chips.conflict && <Status tone="danger" dot>conflict</Status>}
      {chips.archived && <Status tone="neutral" dot>archived</Status>}
    </Flex>
  );
};

export { TestChips };
