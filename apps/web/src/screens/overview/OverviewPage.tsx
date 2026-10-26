/* @layer renderer-app @kind component */
import { ContentHeader, Tabs } from '@drizztdourden08/tessera';
import { usePortfolio } from '../../data/react';
import { PageFrame, StatusNotice, useHashTab } from '../../shell';
import { useTabBadges } from './behavior/useTabBadges';
import { OVERVIEW_TAB_IDS, OVERVIEW_TABS } from './overview.constants';
import { TabBody } from './sub-components/TabBody';

const OverviewPage = () => {
  const { tab, select } = useHashTab(OVERVIEW_TAB_IDS, 'dashboards');
  const { cards, status } = usePortfolio();
  const badges = useTabBadges(tab);
  const tabs = OVERVIEW_TABS.map((item) => ({ ...item, badge: badges.counts[item.id] }));
  return (
    <PageFrame>
      <ContentHeader title="Argus" />
      <StatusNotice status={status} />
      <Tabs tabs={tabs} activeTab={tab} onTabChange={select} hideZero />
      <TabBody tab={tab} cards={cards} status={status} badges={badges} />
    </PageFrame>
  );
};

export { OverviewPage };
