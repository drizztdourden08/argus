/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadActivityTab = () => import('../activity/ActivityTab').then((loaded) => ({ default: loaded.ActivityTab }));

const LazyActivityTab = lazy(loadActivityTab);

export { LazyActivityTab, loadActivityTab };
