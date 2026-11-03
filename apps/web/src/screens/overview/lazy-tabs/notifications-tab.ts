/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyNotificationsTab = lazy(OVERVIEW_TAB_LOADERS.notifications);

export { LazyNotificationsTab };
