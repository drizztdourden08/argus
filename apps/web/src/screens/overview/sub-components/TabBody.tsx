/* @layer renderer-app @kind component */
import { BoardsTab } from '../boards/BoardsTab';
import { LazyAccountsTab, LazyActivityTab, LazyCacheTab, LazyDependenciesTab, LazyNotificationsTab, LazyOverlordTab, LazyProcessesTab, LazyRegistryTab, LazyReleasesTab } from '../lazy-tabs';
import type { TabBodyProps } from '../overview.type';

const TabBody = (props: TabBodyProps) => {
  const { tab, data, view, send, onRail, badges } = props;
  if (tab === 'processes') return <LazyProcessesTab />;
  if (tab === 'accounts') return <LazyAccountsTab />;
  if (tab === 'activity') return <LazyActivityTab freshSince={badges.freshActivity} />;
  if (tab === 'cache') return <LazyCacheTab />;
  if (tab === 'releases') return <LazyReleasesTab />;
  if (tab === 'dependencies') return <LazyDependenciesTab />;
  if (tab === 'registry') return <LazyRegistryTab />;
  if (tab === 'overlord') return <LazyOverlordTab />;
  if (tab === 'notifications') return <LazyNotificationsTab freshSince={badges.freshNotifications} />;
  return <BoardsTab data={data} view={view} send={send} onRail={onRail} />;
};

export { TabBody };
