/* @layer renderer-app @kind component */
import { ContentHeader, Tabs } from '@drizztdourden08/tessera';
import { usePortfolio } from '../../data/react';
import { PageFrame, StatusNotice, useHashTab } from '../../shell';
import { useTabBadges } from './behavior/useTabBadges';
import { OVERVIEW_TAB_IDS, OVERVIEW_TABS } from './overview.constants';
import { ProjectSwitcher } from './sub-components/ProjectSwitcher';
import { TabBody } from './sub-components/TabBody';

const OverviewPage = () => {
  const { tab, select } = useHashTab(OVERVIEW_TAB_IDS, 'dashboards');
  const { cards, status } = usePortfolio();
  const badges = useTabBadges(tab);
  const tabs = OVERVIEW_TABS.map((item) => ({ ...item, badge: badges.counts[item.id], badgeTone: badges.tones[item.id] }));
  return (
    <PageFrame>
      <ContentHeader variant="plain" title="Argus" actions={<ProjectSwitcher cards={cards} />} />
      <StatusNotice status={status} />
      <Tabs tabs={tabs} activeTab={tab} onTabChange={select} hideZero={false} />
      <TabBody tab={tab} cards={cards} status={status} badges={badges} />
    </PageFrame>
  );
};

export { OverviewPage };
