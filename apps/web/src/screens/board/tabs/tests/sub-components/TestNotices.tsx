/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { OpNotice } from '../../kanban/sub-components/OpNotice';
import type { TestNoticesProps } from '../tests-props.type';

const TestNotices = (props: TestNoticesProps) => {
  const { label, notices, ops } = props;
  if (notices.length === 0) return null;
  return (
    <Flex direction="column" gap="sm" role="region" aria-label={label}>
      {notices.map((notice) => <OpNotice key={notice.id} notice={notice} ops={ops} />)}
    </Flex>
  );
};

export { TestNotices };
