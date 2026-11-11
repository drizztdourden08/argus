/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { Suspense, useState } from 'react';
import { usePortfolio } from '../../data/react';
import { LoadingFallback, RecordPaneScope, StatusNotice, useHashTab } from '../../shell';
import { RecordPaneHost } from '../../shell/records/pane';
import { SwitchCountdown, useDeepTab } from './accounts';
import { useBoardEntries } from './boards/behavior/useBoardEntries';
import { useOverviewView } from './behavior/useOverviewView';
import { useTabBadges } from './behavior/useTabBadges';
import { useSendDown } from './boards/behavior/useSendDown';
import { OVERVIEW_HASHES, OVERVIEW_TABS, TAB_ALIASES } from './overview.constants';
import { OverviewHeader } from './sub-components/OverviewHeader';
import { OverviewTabs } from './sub-components/OverviewTabs';
import { TabBody } from './sub-components/TabBody';

const OverviewPage = () => {
  useDeepTab();
  const { tab: hashed, select } = useHashTab(OVERVIEW_HASHES, 'boards');
  const tab = TAB_ALIASES[hashed] ?? hashed;
  const { status } = usePortfolio();
  const badges = useTabBadges(tab);
  const view = useOverviewView();
  const data = useBoardEntries();
  const send = useSendDown();
  const [railShown, setRailShown] = useState(false);
  const tabs = OVERVIEW_TABS.map((item) => ({ ...item, badge: badges.counts[item.id], badgeTone: badges.tones[item.id] }));
  return (
    <RecordPaneScope>
      <Flex direction="column" gap="md">
        <OverviewHeader tab={tab} data={data} view={view} status={status} send={send} railShown={railShown} />
        <StatusNotice status={status} />
        <SwitchCountdown />
        <OverviewTabs tabs={tabs} activeTab={tab} onTabChange={select} />
        <Suspense fallback={<LoadingFallback />}>
          <TabBody tab={tab} data={data} view={view} send={send} onRail={setRailShown} badges={badges} />
        </Suspense>
        <RecordPaneHost />
      </Flex>
    </RecordPaneScope>
  );
};

export { OverviewPage };
