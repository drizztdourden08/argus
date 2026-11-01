/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyBuildsTab = lazy(() => import('../../tabs/builds').then((loaded) => ({ default: loaded.BuildsTab })));

export { LazyBuildsTab };
