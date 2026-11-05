/* @layer renderer-app @kind component */
import { SegmentedControl } from '@drizztdourden08/tessera';
import { Panel } from '../../../shell';
import type { ThreadChoice } from '../../../data/kanban';
import { useCollection } from '../../../data/react';
import { TabFilterBar, TabGroupSort, useTabTools, useTabView } from '../BoardTabsRow';
import { useNow } from '../shared';
import { useFeedActions } from './behavior/useFeedActions';
import { useFeedFilter } from './behavior/useFeedFilter';
import { feedAuthors } from './model/feed-authors';
import { feedFields } from './model/feed-fields';
import { applyFeedFilters } from './model/feed-filters';
import { FEED_SORT_OPTIONS, FEED_VIEW, FEED_VIEW_LABEL, FILTER_OPTIONS } from './OwnerFeed.constants';
import type { OwnerFeedProps } from './OwnerFeed.type';
import { FeedSections } from './sub-components/FeedSections';

const OwnerFeed = (props: OwnerFeedProps) => {
  const { boardKey, work } = props;
  const actions = useFeedActions(boardKey);
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  const now = useNow(true);
  const [filter, setFilter] = useFeedFilter();
  const view = useTabView('activity', FEED_VIEW);
  const shown = applyFeedFilters(work, view.filters, view.direction);
  const count = work.waiting.length;
  const ctx = { boardKey, threads: threads.docs, now, index: work.index, actions };
  useTabTools('activity', [
    {
      id: 'view',
      priority: 4,
      node: <SegmentedControl selectedTone="neutral" size="sm" aria-label={FEED_VIEW_LABEL} value={filter} options={FILTER_OPTIONS} onChange={setFilter} />,
    },
    {
      id: 'sort',
      priority: 5,
      node: <TabGroupSort sort={{ options: FEED_SORT_OPTIONS, order: view.order, direction: view.direction, onChange: view.setSort }} />,
    },
  ]);
  return (
    <Panel title="Activity" note={count > 0 ? `${count} waiting on you` : undefined}>
      <TabFilterBar
        tab="activity"
        boardKey={boardKey}
        fields={feedFields(feedAuthors(work))}
        filters={view.filters}
        patchFilters={view.patchFilters}
        placeholder="Search the activity"
      />
      <FeedSections filter={filter} work={shown} ctx={ctx} />
    </Panel>
  );
};

export { OwnerFeed };
