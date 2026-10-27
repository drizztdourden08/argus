/* @layer renderer-app @kind component */
import { SegmentedControl } from '@drizztdourden08/tessera';
import { Panel } from '../../../shell';
import { useFeedActions } from './behavior/useFeedActions';
import { useFeedFilter } from './behavior/useFeedFilter';
import { FILTER_OPTIONS } from './OwnerFeed.constants';
import type { OwnerFeedProps } from './OwnerFeed.type';
import { FeedSections } from './sub-components/FeedSections';

const OwnerFeed = (props: OwnerFeedProps) => {
  const { boardKey, work, handed } = props;
  const actions = useFeedActions(boardKey, handed);
  const [filter, setFilter] = useFeedFilter();
  const count = work.waiting.length;
  return (
    <Panel
      title="Activity"
      note={count > 0 ? `${count} waiting on you` : undefined}
      tools={<SegmentedControl size="sm" aria-label="Activity view" value={filter} options={FILTER_OPTIONS} onChange={setFilter} />}
    >
      <FeedSections filter={filter} work={work} actions={actions} />
    </Panel>
  );
};

export { OwnerFeed };
