/* @layer renderer-app @kind component */
import { Button, EmptyState, Inline, Stack, Text } from '@drizztdourden08/tessera';
import { FeedTable } from '../../sub-components/FeedTable';
import { NOTHING_MATCHES } from '../activity.constants';
import type { ActivityFeedProps } from '../activity.type';
import { ActivityRow } from './ActivityRow';

const ActivityFeed = (props: ActivityFeedProps) => {
  const { items, shown, freshSince, onMore } = props;
  if (items.length === 0) return <EmptyState message={NOTHING_MATCHES} />;
  const part = items.slice(0, shown);
  return (
    <Stack gap="sm">
      <FeedTable label="Activity">
        {part.map((item) => <ActivityRow key={item.id} item={item} freshSince={freshSince} />)}
      </FeedTable>
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
