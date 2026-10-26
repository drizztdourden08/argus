/* @layer renderer-app @kind component */
import { ActivityTab } from '../activity/ActivityTab';
import { DashboardsTab } from '../dashboards/DashboardsTab';
import { NotificationsTab } from '../notifications/NotificationsTab';
import type { TabBodyProps } from '../overview.type';
import { ProcessesTab } from '../processes/ProcessesTab';

const TabBody = (props: TabBodyProps) => {
  const { tab, cards, status, badges } = props;
  if (tab === 'processes') return <ProcessesTab />;
  if (tab === 'activity') return <ActivityTab freshSince={badges.freshActivity} />;
  if (tab === 'notifications') return <NotificationsTab freshSince={badges.freshNotifications} />;
  return <DashboardsTab cards={cards} status={status} />;
};

export { TabBody };
