/* @layer renderer-app @kind component */
import { LazyActivityTab, LazyCacheTab, LazyDashboardsTab, LazyDependenciesTab, LazyNotificationsTab, LazyProcessesTab, LazyRegistryTab, LazyReleasesTab } from '../lazy-tabs';
import type { TabBodyProps } from '../overview.type';

const TabBody = (props: TabBodyProps) => {
  const { tab, cards, status, badges } = props;
  if (tab === 'processes') return <LazyProcessesTab />;
  if (tab === 'activity') return <LazyActivityTab freshSince={badges.freshActivity} />;
  if (tab === 'cache') return <LazyCacheTab />;
  if (tab === 'releases') return <LazyReleasesTab />;
  if (tab === 'dependencies') return <LazyDependenciesTab />;
  if (tab === 'registry') return <LazyRegistryTab />;
  if (tab === 'notifications') return <LazyNotificationsTab freshSince={badges.freshNotifications} />;
  return <LazyDashboardsTab cards={cards} status={status} />;
};

export { TabBody };
