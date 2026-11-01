/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyActivityTab = lazy(() => import('../../tabs/activity').then((loaded) => ({ default: loaded.ActivityTab })));

export { LazyActivityTab };
