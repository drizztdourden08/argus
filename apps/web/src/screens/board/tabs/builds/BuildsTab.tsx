/* @layer renderer-app @kind component */
import { Callout, EmptyState, Text } from '@drizztdourden08/tessera';
import { useTabView } from '../../BoardTabsRow';
import { useBuildsData } from './behavior/useBuildsData';
import { useRetiredShown } from './behavior/useRetiredShown';
import { EMPTY_HINT, EMPTY_TEXT, NO_MASTER_TEXT, NO_MATCH_TEXT } from './builds.constants';
import type { BuildsTabProps } from './builds-props.type';
import { BUILD_FILTERS, BUILD_GROUP_IDS, BUILD_ORDER_IDS, BUILDS_TAB, buildsView, countsText, groupByItem } from './model';
import { Panel } from '../../../../shell';
import { BuildsLead } from './sub-components/BuildsLead';
import { BuildSections } from './sub-components/BuildSections';
import { RetiredSection } from './sub-components/RetiredSection';
import { BuildsTools } from './BuildsTools';

const BuildsTab = (props: BuildsTabProps) => {
  const { boardKey } = props;
  const tabView = useTabView(BUILDS_TAB, { groups: BUILD_GROUP_IDS, orders: BUILD_ORDER_IDS, group: 'none', order: 'updated', direction: 'desc', filters: BUILD_FILTERS });
  const data = useBuildsData(boardKey);
  const flag = useRetiredShown();
  const { views, runs, threads } = data;
  if (!data.ready) return <Text tone="muted">Loading the builds.</Text>;
  const shown = buildsView(views, { filters: tabView.filters, order: tabView.order, direction: tabView.direction });
  const groups = groupByItem(shown, data.items);
  const sections = (
    <BuildSections groups={groups} group={tabView.group} views={shown.live} boardKey={boardKey} runs={runs} threads={threads} />
  );
  return (
    <Panel title="Builds" note={<BuildsLead counts={countsText(views)} />}>
      <BuildsTools boardKey={boardKey} tabView={tabView} data={data} />
      {!views.hasMaster && <Callout tone="danger">{NO_MASTER_TEXT}</Callout>}
      {views.live.length === 0 && <EmptyState message={EMPTY_TEXT} hint={EMPTY_HINT} />}
      {views.live.length > 0 && shown.live.length === 0 && <Text tone="muted">{NO_MATCH_TEXT}</Text>}
      {shown.live.length > 0 && sections}
      <RetiredSection views={groups.closed} flag={flag} boardKey={boardKey} runs={runs} />
    </Panel>
  );
};

export { BuildsTab };
