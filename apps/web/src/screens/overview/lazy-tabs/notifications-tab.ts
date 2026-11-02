/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadNotificationsTab = () => import('../notifications/NotificationsTab').then((loaded) => ({ default: loaded.NotificationsTab }));

const LazyNotificationsTab = lazy(loadNotificationsTab);

export { LazyNotificationsTab, loadNotificationsTab };
