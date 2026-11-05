/* @layer renderer-app @kind component */
import { EmptyState, Flex } from '@drizztdourden08/tessera';
import { feedTotal } from '../behavior/feed-total';
import type { FeedSectionsProps } from '../OwnerFeed.type';
import { LogFold } from './LogFold';
import { WorkGroups } from './WorkGroups';

const FeedSections = (props: FeedSectionsProps) => {
  const { filter, work, ctx } = props;
  if (filter === 'all' && feedTotal(work) === 0) {
    return <EmptyState message="Nothing here yet." size="sm" />;
  }
  return (
    <Flex direction="column" gap="md">
      <WorkGroups {...props} />
      {filter === 'you' ? null : <LogFold filter={filter} notices={work.notices} lines={work.lines} ctx={ctx} />}
    </Flex>
  );
};

export { FeedSections };
