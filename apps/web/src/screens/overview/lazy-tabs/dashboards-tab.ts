/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyDashboardsTab = lazy(OVERVIEW_TAB_LOADERS.dashboards);

export { LazyDashboardsTab };
