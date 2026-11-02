/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadActivityTab = () => import('../../tabs/activity').then((loaded) => ({ default: loaded.ActivityTab }));

const LazyActivityTab = lazy(loadActivityTab);

export { LazyActivityTab, loadActivityTab };
