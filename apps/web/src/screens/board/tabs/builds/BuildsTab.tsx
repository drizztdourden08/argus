/* @layer renderer-app @kind component */
import { EmptyState, Text } from '@drizztdourden08/tessera';
import { useTabView } from '../../BoardTabsRow';
import { SECOND_TICK_MS, TabPanel, useNow, useStoredFlag, useTabLayout } from '../../shared';
import { useBuildsData } from './behavior/useBuildsData';
import { EMPTY_TEXT, LAYOUT_IDS, NO_MATCH_TEXT, RETIRED_KEY } from './builds.constants';
import type { BuildsTabProps } from './builds-props.type';
import { BUILD_FILTERS, BUILD_GROUP_IDS, BUILD_ORDER_IDS, BUILDS_TAB, buildsView, buildStats, groupByItem, liveRuns } from './model';
import { BuildRunsSection } from './sub-components/BuildRunsSection';
import { BuildSections } from './sub-components/BuildSections';
import { RetiredSection } from './sub-components/RetiredSection';
import { BuildsTools } from './BuildsTools';
import './BuildsTab.css';

const BuildsTab = (props: BuildsTabProps) => {
  const { boardKey } = props;
  const tabView = useTabView(BUILDS_TAB, { groups: BUILD_GROUP_IDS, orders: BUILD_ORDER_IDS, group: 'none', order: 'updated', direction: 'desc', filters: BUILD_FILTERS });
  const data = useBuildsData(boardKey);
  const [layout, setLayout] = useTabLayout(BUILDS_TAB, boardKey, LAYOUT_IDS, 'normal');
  const [showClosed, setShowClosed] = useStoredFlag(RETIRED_KEY);
  const { views, runs, threads } = data;
  const running = liveRuns(runs);
  const now = useNow(running.length > 0, SECOND_TICK_MS);
  if (!data.ready) return <Text tone="muted">Loading the builds.</Text>;
  const shown = buildsView(views, { filters: tabView.filters, order: tabView.order, direction: tabView.direction });
  const groups = groupByItem(shown, data.items);
  const compact = layout === 'compact';
  const closed = { count: views.retired.length, shown: showClosed, onToggle: () => setShowClosed(!showClosed) };
  return (
    <>
      <BuildsTools boardKey={boardKey} tabView={tabView} data={data} layout={{ value: layout, onChange: setLayout }} closed={closed} />
      <TabPanel label="Builds" stats={buildStats(views, data.groups)}>
        <BuildRunsSection runs={running} boardKey={boardKey} now={now} />
        {views.live.length === 0 && <EmptyState message={EMPTY_TEXT} />}
        {views.live.length > 0 && shown.live.length === 0 && <Text tone="muted">{NO_MATCH_TEXT}</Text>}
        {shown.live.length > 0 && (
          <BuildSections groups={groups} group={tabView.group} views={shown.live} boardKey={boardKey} runs={runs} threads={threads} compact={compact} />
        )}
        {showClosed && <RetiredSection views={groups.closed} boardKey={boardKey} runs={runs} compact={compact} />}
      </TabPanel>
    </>
  );
};

export { BuildsTab };
