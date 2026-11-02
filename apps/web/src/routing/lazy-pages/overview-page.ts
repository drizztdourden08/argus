/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadOverviewPage = () => import('../../screens/overview/OverviewPage').then((loaded) => ({ default: loaded.OverviewPage }));

const LazyOverviewPage = lazy(loadOverviewPage);

export { LazyOverviewPage, loadOverviewPage };
