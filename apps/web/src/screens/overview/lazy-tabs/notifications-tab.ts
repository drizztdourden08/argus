/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyNotificationsTab = lazy(() => import('../notifications/NotificationsTab').then((loaded) => ({ default: loaded.NotificationsTab })));

export { LazyNotificationsTab };
