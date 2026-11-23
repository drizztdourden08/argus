/* @layer renderer-app @kind component */
import { EmptyState } from '@drizztdourden08/tessera';
import { Panel } from '../../../shell';
import type { ThreadChoice } from '../../../data/kanban';
import { useCollection } from '../../../data/react';
import { TabFilterBar, TabGroupSort, useTabTools, useTabView } from '../BoardTabsRow';
import { useNow } from '../shared';
import { useFeedActions } from './behavior/useFeedActions';
import { askCards } from './behavior/ask-cards';
import { feedAuthors } from './behavior/feed-authors';
import { feedFields } from './behavior/feed-fields';
import { applyFeedFilters } from './behavior/feed-filters';
import { FEED_SORT_OPTIONS, FEED_VIEW } from './OwnerFeed.constants';
import type { OwnerFeedProps } from './OwnerFeed.type';
import { AskGroup } from './sub-components/AskGroup';

const OwnerFeed = (props: OwnerFeedProps) => {
  const { boardKey, work } = props;
  const actions = useFeedActions(boardKey);
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  const now = useNow(true);
  const view = useTabView('activity', FEED_VIEW);
  const asks = askCards(work.waiting);
  const shown = applyFeedFilters(asks, view.filters, view.direction);
  const ctx = { boardKey, threads: threads.docs, now, index: work.index, actions };
  useTabTools('activity', [
    {
      id: 'sort',
      priority: 5,
      node: <TabGroupSort sort={{ options: FEED_SORT_OPTIONS, order: view.order, direction: view.direction, onChange: view.setSort }} />,
    },
  ]);
  return (
    <Panel title="Activity" note={asks.length > 0 ? `${asks.length} waiting on you` : undefined}>
      <TabFilterBar
        tab="activity"
        boardKey={boardKey}
        fields={feedFields(feedAuthors(asks))}
        filters={view.filters}
        patchFilters={view.patchFilters}
        placeholder="Search the questions and decisions"
      />
      {shown.length === 0 ? <EmptyState message="No questions or decisions waiting." size="sm" /> : <AskGroup cards={shown} ctx={ctx} />}
    </Panel>
  );
};

export { OwnerFeed };
