/* @layer renderer-app @kind component */
import { Icon, Tabs } from '@drizztdourden08/tessera';
import type { TabItem } from '@drizztdourden08/tessera';
import { DEFAULT_TAB_ICON, TAB_ICONS } from '../BoardTabsRow.constants';
import type { BoardTabsProps } from '../BoardTabsRow.type';

const withIcon = (tab: TabItem): TabItem => ({ ...tab, icon: tab.icon ?? <Icon name={TAB_ICONS[tab.id] ?? DEFAULT_TAB_ICON} /> });

const BoardTabs = (props: BoardTabsProps) => {
  const { tabs, activeTab, onTabChange, tight } = props;
  return (
    <Tabs tabs={tabs.map(withIcon)} activeTab={activeTab} size="lg" selectedTone="neutral" badgeFill="solid" iconOnly={tight} onTabChange={onTabChange} />
  );
};

export { BoardTabs };
