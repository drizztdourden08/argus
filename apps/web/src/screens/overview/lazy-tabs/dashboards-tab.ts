/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyDashboardsTab = lazy(() => import('../dashboards/DashboardsTab').then((loaded) => ({ default: loaded.DashboardsTab })));

export { LazyDashboardsTab };
