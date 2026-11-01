/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyOverviewPage = lazy(() => import('../../screens/overview/OverviewPage').then((loaded) => ({ default: loaded.OverviewPage })));

export { LazyOverviewPage };
