/* @layer renderer-app @kind component */
import { Flex, Tabs } from '@drizztdourden08/tessera';
import { Suspense } from 'react';
import { usePortfolio } from '../../data/react';
import { LoadingFallback, RecordPaneScope, StatusNotice, useHashTab } from '../../shell';
import { RecordPaneHost } from '../../shell/records/pane';
import { useTabBadges } from './behavior/useTabBadges';
import { OVERVIEW_TAB_IDS, OVERVIEW_TABS } from './overview.constants';
import { OverviewHeader } from './sub-components/OverviewHeader';
import { TabBody } from './sub-components/TabBody';

const OverviewPage = () => {
  const { tab, select } = useHashTab(OVERVIEW_TAB_IDS, 'dashboards');
  const { cards, status } = usePortfolio();
  const badges = useTabBadges(tab);
  const tabs = OVERVIEW_TABS.map((item) => ({ ...item, badge: badges.counts[item.id], badgeTone: badges.tones[item.id] }));
  return (
    <RecordPaneScope>
      <Flex direction="column" gap="md">
        <OverviewHeader />
        <StatusNotice status={status} />
        <Tabs tabs={tabs} activeTab={tab} onTabChange={select} hideZero={false} size="lg" selectedTone="neutral" badgeFill="solid" />
        <Suspense fallback={<LoadingFallback />}>
          <TabBody tab={tab} cards={cards} status={status} badges={badges} />
        </Suspense>
        <RecordPaneHost />
      </Flex>
    </RecordPaneScope>
  );
};

export { OverviewPage };
