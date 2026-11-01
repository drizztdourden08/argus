/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyActivityTab = lazy(() => import('../activity/ActivityTab').then((loaded) => ({ default: loaded.ActivityTab })));

export { LazyActivityTab };
