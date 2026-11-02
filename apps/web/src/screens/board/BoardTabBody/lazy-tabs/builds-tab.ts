/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadBuildsTab = () => import('../../tabs/builds').then((loaded) => ({ default: loaded.BuildsTab }));

const LazyBuildsTab = lazy(loadBuildsTab);

export { LazyBuildsTab, loadBuildsTab };
