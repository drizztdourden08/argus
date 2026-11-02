/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadReleasesTab = () => import('../releases/ReleasesTab').then((loaded) => ({ default: loaded.ReleasesTab }));

const LazyReleasesTab = lazy(loadReleasesTab);

export { LazyReleasesTab, loadReleasesTab };
