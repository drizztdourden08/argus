/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { OpNoticesProps } from '../kanban-props.type';
import { OpNoticeCallout } from '../../../shared';
import { PurgeNoticeView } from './PurgeNoticeView';

const OpNotices = (props: OpNoticesProps) => {
  const { ops } = props;
  if (ops.notices.length === 0 && ops.purges.length === 0) return null;
  return (
    <Flex direction="column" gap="sm" role="region" aria-label="Changes the board did not take">
      {ops.notices.map((notice) => (
        <OpNoticeCallout key={notice.id} notice={notice} ops={ops} />
      ))}
      {ops.purges.map((notice) => (
        <PurgeNoticeView key={notice.id} notice={notice} ops={ops} />
      ))}
    </Flex>
  );
};

export { OpNotices };
