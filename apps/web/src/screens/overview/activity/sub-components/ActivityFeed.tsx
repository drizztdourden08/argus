/* @layer renderer-app @kind component */
import { Button, EmptyState, Inline, ListItemList, Stack, Text } from '@drizztdourden08/tessera';
import { ACTIVITY_HEADER, ACTIVITY_SHAPE, NOTHING_MATCHES } from '../activity.constants';
import type { ActivityFeedProps } from '../activity.type';
import { ActivityRow } from './ActivityRow';

const ActivityFeed = (props: ActivityFeedProps) => {
  const { items, shown, freshSince, onMore } = props;
  if (items.length === 0) return <EmptyState message={NOTHING_MATCHES} />;
  const part = items.slice(0, shown);
  return (
    <Stack gap="sm">
      <ListItemList label="Activity" header={ACTIVITY_HEADER} shape={ACTIVITY_SHAPE} count={items.length}>
        {part.map((item) => <ActivityRow key={item.id} item={item} freshSince={freshSince} />)}
      </ListItemList>
      {items.length > part.length && (
        <Inline gap="sm" align="center">
          <Button onClick={onMore}>Show more</Button>
          <Text tone="muted">{`${part.length} of ${items.length} shown`}</Text>
        </Inline>
      )}
    </Stack>
  );
};

export { ActivityFeed };
