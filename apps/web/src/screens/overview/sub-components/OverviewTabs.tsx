/* @layer renderer-app @kind component */
import { Icon, OverflowRow, Tabs } from '@drizztdourden08/tessera';
import type { OverflowRowItem, TabItem } from '@drizztdourden08/tessera';
import { TAB_ICONS } from '../overview.constants';
import type { OverviewTabsProps } from '../overview.type';

const withIcon = (tab: TabItem): TabItem => ({ ...tab, icon: tab.icon ?? <Icon name={TAB_ICONS[tab.id] ?? 'folder'} /> });

const OverviewTabs = (props: OverviewTabsProps) => {
  const { tabs, activeTab, onTabChange } = props;
  const items: OverflowRowItem[] = [
    {
      id: 'tabs',
      pinned: true,
      node: ({ tight }) => (
        <Tabs tabs={tabs.map(withIcon)} activeTab={activeTab} onTabChange={onTabChange} hideZero={false} size="lg" selectedTone="neutral" badgeFill="solid" iconOnly={tight} />
      ),
    },
  ];
  return <OverflowRow items={items} />;
};

export { OverviewTabs };
