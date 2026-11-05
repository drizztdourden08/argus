/* @layer renderer-app @kind component */
import { SegmentedControl } from '@drizztdourden08/tessera';
import { Panel } from '../../../shell';
import type { ThreadChoice } from '../../../data/kanban';
import { useCollection } from '../../../data/react';
import { TabFilters } from '../BoardTabsRow';
import { useNow } from '../shared';
import { useFeedActions } from './behavior/useFeedActions';
import { useFeedFilter } from './behavior/useFeedFilter';
import { FILTER_OPTIONS } from './OwnerFeed.constants';
import type { OwnerFeedProps } from './OwnerFeed.type';
import { FeedSections } from './sub-components/FeedSections';

const OwnerFeed = (props: OwnerFeedProps) => {
  const { boardKey, work } = props;
  const actions = useFeedActions(boardKey);
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  const now = useNow(true);
  const [filter, setFilter] = useFeedFilter();
  const count = work.waiting.length;
  const ctx = { boardKey, threads: threads.docs, now, index: work.index, actions };
  return (
    <Panel
      title="Activity"
      note={count > 0 ? `${count} waiting on you` : undefined}
      tools={(
        <TabFilters tab="activity" count={filter === 'all' ? 0 : 1}>
          <SegmentedControl selectedTone="neutral" size="sm" aria-label="Activity view" value={filter} options={FILTER_OPTIONS} onChange={setFilter} />
        </TabFilters>
      )}
    >
      <FeedSections filter={filter} work={work} ctx={ctx} />
    </Panel>
  );
};

export { OwnerFeed };
