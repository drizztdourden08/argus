/* @layer renderer-app @kind component */
import { Card, Flex, SegmentedControl } from '@drizztdourden08/tessera';
import { useFeedActions } from './behavior/useFeedActions';
import { useFeedFilter } from './behavior/useFeedFilter';
import { FILTER_OPTIONS } from './OwnerFeed.constants';
import type { OwnerFeedProps } from './OwnerFeed.type';
import { FeedSections } from './sub-components/FeedSections';

const OwnerFeed = (props: OwnerFeedProps) => {
  const { boardKey, work } = props;
  const actions = useFeedActions(boardKey);
  const [filter, setFilter] = useFeedFilter();
  const count = work.waiting.length;
  return (
    <Card title="Activity" subtitle={count > 0 ? `${count} waiting on you` : undefined} level={2}>
      <Flex direction="column" gap="md">
        <SegmentedControl label="Activity view" value={filter} options={FILTER_OPTIONS} onChange={setFilter} />
        <FeedSections filter={filter} work={work} actions={actions} />
      </Flex>
    </Card>
  );
};

export { OwnerFeed };
