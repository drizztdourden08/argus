/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadDashboardsTab = () => import('../dashboards/DashboardsTab').then((loaded) => ({ default: loaded.DashboardsTab }));

const LazyDashboardsTab = lazy(loadDashboardsTab);

export { LazyDashboardsTab, loadDashboardsTab };
