/* @layer renderer-app @kind component */
import { Card, EmptyState, Flex } from '@drizztdourden08/tessera';
import { NO_NEEDS } from '../feed-model';
import { useOwnerPost } from '../owner-posts';
import type { WorkItemsProps } from './WorkItems.type';
import { WorkItemCard } from './sub-components/WorkItemCard';

const WorkItems = (props: WorkItemsProps) => {
  const { boardKey, items, needs } = props;
  const poster = useOwnerPost(boardKey);
  return (
    <Card title="Work items" subtitle={`${items.length} open`} level={2}>
      {items.length === 0 ? (
        <EmptyState message="No open work item." size="sm" />
      ) : (
        <Flex direction="column" gap="xs">
          {items.map((item) => (
            <WorkItemCard key={item.id} item={item} need={needs.get(item.id) ?? NO_NEEDS} poster={poster} />
          ))}
        </Flex>
      )}
    </Card>
  );
};

export { WorkItems };
