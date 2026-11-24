/* @layer renderer-app @kind component */
import { Flex, Status, Tag, Text } from '@drizztdourden08/tessera';
import { activityTone, nonEmpty } from '../../shared';
import { isLeadRole } from '../../../../data/content';
import type { ThreadEyebrowProps } from '../ThreadDrawer.type';

const ThreadEyebrow = (props: ThreadEyebrowProps) => {
  const { thread, counts } = props;
  return (
    <Flex gap="xs" align="center" wrap>
      <Text as="span" variant="caption" weight={600} mono>{thread.id}</Text>
      <Tag>{thread.status}</Tag>
      <Status tone={activityTone(thread.activity)} dot>{nonEmpty(thread.activity, 'unknown')}</Status>
      {isLeadRole(thread.role) && <Tag variant="urgency" color="info">coord</Tag>}
      {thread.archivedAt && <Tag>archived</Tag>}
      {counts.questions > 0 && <Tag variant="urgency" color="warning">{`${counts.questions} open`}</Tag>}
      {counts.toPickUp > 0 && <Tag variant="urgency" color="warning">{`${counts.toPickUp} to pick up`}</Tag>}
    </Flex>
  );
};

export { ThreadEyebrow };
