/* @layer renderer-app @kind component */
import { Icon, Tabs } from '@drizztdourden08/tessera';
import type { TabItem } from '@drizztdourden08/tessera';
import { PANE_TABS } from './pane.constants';
import type { PaneTabsProps } from './pane.type';

const PaneTabs = (props: PaneTabsProps) => {
  const { data, tabs, active, onSelect } = props;
  const items: TabItem[] = tabs.map((id) => {
    const tab = PANE_TABS[id];
    const badge = tab?.badge?.(data);
    return { id, label: tab?.label ?? id, icon: tab && <Icon name={tab.icon} />, badge: badge?.value, badgeTone: badge?.tone, badgeTitle: badge?.title };
  });
  return <Tabs tabs={items} activeTab={active} onTabChange={onSelect} hideZero />;
};

export { PaneTabs };
