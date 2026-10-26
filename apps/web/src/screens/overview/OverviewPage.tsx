/* @layer renderer-app @kind component */
import { ContentHeader, EmptyState, Tabs } from '@drizztdourden08/tessera';
import { usePortfolio } from '../../data/react';
import { PageFrame, StatusNotice, useHashTab } from '../../shell';
import { OVERVIEW_TAB_IDS, OVERVIEW_TABS } from './overview.constants';
import { ProjectGrid } from './sub-components/ProjectGrid';

const OverviewPage = () => {
  const { tab, select } = useHashTab(OVERVIEW_TAB_IDS, 'dashboards');
  const { cards, status } = usePortfolio();
  return (
    <PageFrame>
      <ContentHeader title="Argus" />
      <StatusNotice status={status} />
      <Tabs tabs={OVERVIEW_TABS} activeTab={tab} onTabChange={select} />
      {tab === 'dashboards' ? <ProjectGrid cards={cards} /> : <EmptyState message="This tab is not built yet." />}
    </PageFrame>
  );
};

export { OverviewPage };
